//! Generic Push-Pull Solver.
//!
//! This crate implements a generic solver for anything that can have a clear dependency graph.
//! The implementation is a mix of push (eager) and pull (lazy) architectures with user-driven
//! recursion.
//!
//! Functionality is centered on the [`Solver`] struct. Users record all *fragments* they want to
//! evaluate and only those. Fragments are represented by an integral [`FragmentId`], but what
//! *is* a fragment is arbitrary and the solver does not care. It may represent a variable, an
//! action, an object, or anything else.
//!
//! Users must also implement the [`Problem`] trait, which defines a dependency graph and an
//! interface for evaluating fragments that the solver finds are both solvable and required. This
//! dependency graph does not need to be complete or explicit, as long as implementors can return
//! the direct dependencies of specified fragments as the solver explores the dependency graph.
//!
//! [`Solver::run`] and [`Solver::step`] will incrementally explore the depedency graph and call
//! [`Problem::evaluate`] on fragments that have all of its dependencies met.
//!
//! In the end, all requested fragments will either have been evaluated or will be proven to be
//! part of a dependency cycle. The user may choose to report cycles as errors, or break them with
//! [`Solver::assume_evaluated`] or [`Solver::clone_with_evaluation_assumptions`]. See also
//! [`Solver::status`].
//!
//! [`Solver::punted_iter`] will return an iterator yielding all fragments that have been *punted*
//! so far. A punted fragment is one that has been considered for evaluation but its dependencies
//! haven't been met yet. If the solver is done, punted fragments must be part of at least one
//! cycle.
//!
//! # Concurrency
//!
//! [`Solver`] is fully asynchronous but the core algorithm is not parallel at the moment. Running
//! multiple [`Solver::step`] concurrently or calling [`Solver::run`] with `concurrency > 1` is
//! safe but will not make the solver itself run faster. What this does allow is for multiple
//! [`Problem::direct_dependencies`] and [`Problem::evaluate`] calls to run concurrently.
//!
//! # Internals
//!
//! [`Solver`] implements a hybrid push-pull architecture. Fragments are only evaluated if needed
//! (pull, lazy evaluation), but instead of evaluating dependencies recursively, this process will
//! only evaluate fragments that already have all of its *direct* dependencies met. If that's not
//! the case, the fragment will be *punted*: stored away and only considered again if *all* its
//! dependencies are met sometime in the future.
//!
//! On the other hand, if a fragment is successfully evaluated, punted fragments that depend on it
//! will be evaluated eagerly (push) if all other dependencies have also been evaluated.
//!
//! This architecture has two major advantages:
//!
//! - It is lazy. Only fragments that are explicitly requested to be evaluated, and the fragments
//!   those depend on, will be evaluated. And never more than once.
//! - There is no need to explicitly detect nor handle cycles, unlike both pure push and pure
//!   pull. Fragments that are part of cycles will naturally be punted and never considered again.
//!   Unless the cycle is explicitly broken with [`Solver::assume_evaluated`] or
//!   [`Solver::clone_with_evaluation_assumptions`]. This enables a much simpler implementation.

#![cfg_attr(not(feature = "std"), no_std)]

// Only used when testing
#[cfg(test)]
macro_rules! family_cfg {
    (for $name:literal; $($item:item)*) => {
        $(
            #[cfg(target_family = $name)]
            $item
        )*
    };
    (for !$name:literal; $($item:item)*) => {
        $(
            #[cfg(not(target_family = $name))]
            $item
        )*
    };
}

macro_rules! feature_cfg {
    (for $name:literal; $($item:item)*) => {
        $(
            #[cfg(feature = $name)]
            $item
        )*
    };
    (for !$name:literal; $($item:item)*) => {
        $(
            #[cfg(not(feature = $name))]
            $item
        )*
    };
}

use crate::reexported::{iter, Box, Map, Mutex, NonZeroUsize, Set, Vec};
use async_trait::async_trait;
use derive_more::{From, Into};
use futures::stream::{FuturesUnordered, StreamExt};

pub mod reexported;

#[cfg(all(feature = "js-bindings", target_family = "wasm"))]
mod js;

#[cfg(test)]
mod test;

/// Trait implemented by objects that define a specific problem to be solved by the [`Solver`].
///
/// Use [`mod@async_trait`] to implement this trait.
#[async_trait]
pub trait Problem {
    /// Error type for [`Problem::evaluate`].
    type Error;

    /// Fill `dependencies` with the direct dependencies of `id`. The output vector is guaranteed
    /// to be empty when this method is called.
    async fn direct_dependencies(
        &self,
        id: FragmentId,
        dependecies: &mut Vec<FragmentId>,
    );

    /// Called by the solver to signal that a fragment has had all of its dependencies evaluated.
    /// Thus, the fragment should be evaluated too.
    ///
    /// See [`Solver::run`] and [`Solver::step`] on how evaluation failures are handled.
    ///
    /// This method is never called more than once with the same fragment.
    async fn evaluate(&self, id: FragmentId) -> Result<(), Self::Error>;
}

/// ID of a fragment.
// TODO: allow `Problem` implementors to define their own ID type
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into,
)]
pub struct FragmentId(pub usize);

/// Hybrid push-pull solver.
pub struct Solver<P> {
    state: Mutex<State>,
    // This is a scratch vector we store here to reduce allocations
    dependencies: Mutex<Vec<FragmentId>>,
    problem_instance: P,
}

// POD struct
struct State {
    // TODO: these should be an intrusive copy-on-write to make cloning and testing alternatives
    // cheap
    to_solve: Set<FragmentId>,
    pending_on: Map<FragmentId, Vec<FragmentId>>,
    punted: Map<FragmentId, usize>,
    solved: Set<FragmentId>,
}

impl<P> Solver<P> {
    /// Create a new [`Solver`] instance for a [`Problem`].
    pub fn new(problem_instance: P) -> Self {
        Self {
            state: Mutex::new(State {
                to_solve: Set::new(),
                pending_on: Map::new(),
                punted: Map::new(),
                solved: Set::new(),
            }),
            dependencies: Mutex::new(Vec::new()),
            problem_instance,
        }
    }

    /// Consume `self` and return the wrapped [`Problem`] instance.
    pub fn into_problem_instance(self) -> P {
        self.problem_instance
    }

    /// Get the current [`Status`] of the solver.
    pub async fn status(&self) -> Status {
        let state = self.state.lock().await;

        if state.to_solve.is_empty() {
            if state.punted.is_empty() {
                Status::Done
            } else {
                Status::DoneWithCycles
            }
        } else {
            Status::Pending
        }
    }

    /// Enqueue a fragment to be solved.
    ///
    /// Only fragments enqueued through this method and their transitive dependencies will be
    /// considered for evaluation.
    pub async fn enqueue_fragment(&self, id: FragmentId) -> &Self {
        self.state.lock().await.to_solve.insert(id);

        self
    }

    /// Get an interator to all fragments that are currently punted. Interpretation of punted
    /// fragments depends on the current [status](Solver::status):
    ///
    /// - [`Status::Pending`]: fragments are pending on dependencies.
    /// - [`Status::DoneWithCycles`]: fragments are part of one or more cycles.
    /// - [`Status::Done`]: the returned iterator will be empty.
    pub async fn punted_iter(&self) -> Vec<FragmentId> {
        self.state.lock().await.punted.keys().copied().collect()
    }
}

impl<P> Solver<P>
where
    P: Problem,
{
    /// Assume the given fragment is already evaluated.
    pub async fn assume_evaluated(&self, id: FragmentId) -> &Self {
        self.mark_solved(id, &mut *self.state.lock().await);

        self
    }

    /* TODO: rethink about cloning in general
    /// Create a clone of `self` that assumes some fragments are already evaluated.
    ///
    /// This method is useful for trying out assumptions that may need to be discarted.
    pub async fn clone_with_evaluation_assumptions<A>(
        &self,
        assume_evaluated: A,
    ) -> Self
    where
        A: IntoIterator<Item = FragmentId>,
        P: Clone,
    {
        let clone = self.clone();
        for id in assume_evaluated {
            clone.assume_evaluated(id).await;
        }

        clone
    }
    */

    /// Run the solver until all enqueued fragments and their transitive dependencies are either
    /// solved or proven to be part of at least one cycle. See the module docs for the limitations
    /// when `concurrency > 1`.
    ///
    /// Returns an interator with all fragments that are part of at least one cycle, if any. See
    /// [`Solver::punted_iter`].
    ///
    /// Returns an error if any evaluation returns an error.
    ///
    /// # Known Issues
    ///
    /// - If [`Solver::enqueue_fragment`] is called while [`Solver::run`] is executing, those new
    ///   fragments may not be solved.
    /// - If [`Solver::run`] returns with an error, the [`Solver`] may be left in an inconsistent
    ///   state.
    pub async fn run(
        &self,
        concurrency: NonZeroUsize,
    ) -> Result<Vec<FragmentId>, P::Error> {
        let mut steps = iter::repeat_with(|| self.step())
            .take(concurrency.into())
            .collect::<FuturesUnordered<_>>();
        loop {
            // Run a `parallelism` number of `step`s until one of them errors out or we evaluate
            // all fragments
            match steps.next().await.unwrap() {
                Ok(false) => break,
                Ok(true) => steps.push(self.step()),
                Err(err) => return Err(err),
            }
        }
        while let Some(res) = steps.next().await {
            // Make sure all pending `step`s are evaluated to completion
            if let Err(err) = res {
                return Err(err);
            }
        }

        Ok(self.punted_iter().await)
    }

    /// Run a single solver step for a single fragment.
    ///
    /// Returns `false` if there are no more fragments that can be evaluated.
    ///
    /// Returns an error if [`Problem::evaluate`] was called and evaluation returned an error.
    ///
    /// # Known Issues
    ///
    /// - If [`Solver::step`] is not run to completion the [`Solver`] may be left in an
    ///   inconsistent state.
    pub async fn step(&self) -> Result<bool, P::Error> {
        let item = {
            let mut state = self.state.lock().await;

            state
                .to_solve
                .iter()
                .next()
                .copied()
                .map(|x| state.to_solve.take(&x).unwrap())
        };

        match item {
            Some(id) => {
                let mut dependencies = self.dependencies.lock().await;
                dependencies.clear();
                self.problem_instance
                    .direct_dependencies(id, &mut dependencies)
                    .await;
                let mut state = self.state.lock().await;
                dependencies.retain(|x| !state.solved.contains(x));

                if dependencies.is_empty() {
                    // Drop all locks before calling `evaluate`to allow other calls to `step` to
                    // progress while `evaluate` is running. And we only need to lock `self.state`
                    // again if `evaluate` is successful
                    drop(dependencies);
                    drop(state);

                    match self.problem_instance.evaluate(id).await {
                        Ok(()) => {
                            // TODO: take a deeper look here to make sure there are no possible
                            // race condition between dropping the state lock and locking it again
                            // here
                            self.mark_solved(id, &mut *self.state.lock().await);

                            Ok(true)
                        }
                        Err(err) => Err(err),
                    }
                } else {
                    self.mark_punted(id, &dependencies, &mut state);

                    Ok(true)
                }
            }
            None => Ok(false),
        }
    }

    fn mark_solved(&self, id: FragmentId, state: &mut State) {
        state.solved.insert(id);

        if let Some(dependents) = state.pending_on.remove(&id) {
            for dependent in dependents {
                if *state.punted.get(&dependent).unwrap() == 1 {
                    state.punted.remove(&dependent);
                    state.to_solve.insert(dependent);
                } else {
                    *state.punted.get_mut(&dependent).unwrap() -= 1;
                }
            }
        }
    }

    fn mark_punted(
        &self,
        id: FragmentId,
        dependencies: &[FragmentId],
        state: &mut State,
    ) {
        state.punted.insert(id, dependencies.len());

        for dependency in dependencies.iter().copied() {
            if dependency != id
                && !state.solved.contains(&dependency)
                && !state.punted.contains_key(&dependency)
            {
                state.to_solve.insert(dependency);
            }
            state.pending_on.entry(dependency).or_default().push(id);
        }
    }
}

/// Current status of a [`Solver`] instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    /// All fragments have been successfully evaluated and no cycles were found.
    Done,

    /// All fragments that could be evaluated were evaluated, but some fragments were part of at
    /// least one dependency cycle and thus could not be evaluated.
    DoneWithCycles,

    /// The solver is still running and there are still fragments that may be evaluated.
    Pending,
}
