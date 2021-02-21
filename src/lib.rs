//! Generic Push-Pull Solver.
//!
//! This crate implements a generic solver for anything that can have a clear
//! dependency graph. The implementation is a mix of push (eager) and pull
//! (lazy) architectures with user-driven recursion.
//!
//! Functionality is centered on the [`Solver`] struct. Users records all
//! *fragments* they want to evaluate and only those.  What *is* a fragment is
//! arbitrary and the solver does not care. Users must also implement the
//! [`Problem`] trait, which defines a dependency graph and an interface for
//! evaluating fragments that the solver finds are both solvable and needed.
//!
//! [`Solver::run`] and [`Solver::step`] will incrementally explore the
//! depedency graph, calling [`Problem::evaluate`] on fragments that have all
//! of its dependencies met.
//!
//! In the end, all requested fragments will either have been evaluated or some
//! of those will have been permanently punted due to being a part of a
//! dependency cycle. The user may choose to report cycles as errors, or break
//! them with [`Solver::assume_evaluated`] or
//! [`Solver::clone_with_evaluation_assumptions`]. See [`Solver::status`].
//!
//! [`Solver::punted_iter`] will return an iterator yielding all fragments that
//! have been *punted* so far. A punted fragment is one that has been
//! considered for evaluation but its dependencies haven't been met yet. If the
//! solver is done, the only punted fragments are those that are part of a
//! cycle.
//!
//! # Internals
//!
//! [`Solver`] implements a hybrid push-pull architecture. This means that
//! fragments are only evaluated if needed (pull, lazy evaluation), but instead
//! of evaluating dependencies recursively, this process will only evaluate
//! fragments that already have all of its dependencies met. If that's not the
//! case, the fragment will be *punted*: stored away and only considered again
//! if all its dependencies are met sometime in the future.
//!
//! On the other hand, if a fragment is successfully evaluated, punted
//! fragments that depend on it may be evaluated eagerly (push).
//!
//! This architecture has two major advantages:
//!
//! - It is lazy. Only fragments that are explicitly requested to be evaluated,
//!   and the fragments those depend on, will be evaluated. And never more than
//!   once.
//! - There is no need to explicitly detect nor handle cycles. Fragments that
//!   are part of cycles will naturally be punted and never considered again
//!   unless the cycle is explicitly broken.

#![cfg_attr(not(std), no_std)]

extern crate alloc;

use derive_more::{From, Into};

#[cfg(std)]
use std::collections::{HashMap, HashSet};

#[cfg(not(std))]
use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};
#[cfg(not(std))]
use core::iter::{IntoIterator, Iterator};

#[cfg(std)]
type Map<K, V> = HashMap<K, V>;
#[cfg(std)]
type Set<T> = HashSet<T>;

#[cfg(not(std))]
type Map<K, V> = BTreeMap<K, V>;
#[cfg(not(std))]
type Set<T> = BTreeSet<T>;

/// Trait implemented by objects that define a specific problem to be solved by
/// the [`Solver`].
pub trait Problem {
    /// Error type for [`Problem::evaluate`].
    type Error;

    /// Fill `dependencies` with the direct dependencies of `id`. The output
    /// vector is guaranteed to be empty when this method is called.
    ///
    /// This method is never called more than once with the same fragment.
    fn direct_dependencies(
        &self,
        id: FragmentId,
        dependecies: &mut Vec<FragmentId>,
    );

    /// Called by the solver to signal that a fragment has had all of its
    /// dependencies evaluated and thus can now be evaluated too.
    ///
    /// See [`Solver::run`] and [`Solver::step`] on how evaluation failures are
    /// handled.
    ///
    /// This method is never called more than once with the same fragment.
    fn evaluate(&mut self, id: FragmentId) -> Result<(), Self::Error>;
}

/// ID of a fragment.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into,
)]
pub struct FragmentId(pub usize);

/// Solver for a specific [`Problem`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Solver<P> {
    // TODO: these should be copy-on-write to make cloning cheap
    to_solve: Set<FragmentId>,
    pending_on: Map<FragmentId, Vec<FragmentId>>,
    punted: Map<FragmentId, usize>,
    solved: Set<FragmentId>,

    // This is a scratch vector we store here to reduce allocations
    dependencies: Vec<FragmentId>,

    problem: P,
}

impl<P> Solver<P> {
    /// Create a new [`Solver`] for a [`Problem`].
    pub fn new(problem: P) -> Self {
        Self {
            to_solve: Set::new(),
            pending_on: Map::new(),
            punted: Map::new(),
            solved: Set::new(),

            dependencies: Vec::new(),

            problem,
        }
    }

    /// Get the current [`Status`] of the solver.
    pub fn status(&self) -> Status {
        if self.to_solve.is_empty() {
            if self.punted.is_empty() {
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
    /// Only fragments enqueued through this method and their transitive
    /// dependencies will be considered for evaluation.
    pub fn enqueue_fragment(&mut self, id: FragmentId) -> &mut Self {
        self.to_solve.insert(id);

        self
    }

    /// Get an interator to all fragments that are currently punted.
    /// Interpretation of punted fragments depends on the current
    /// [status](Solver::status):
    ///
    /// - [`Status::Pending`]: fragments that are pending on dependencies.
    /// - [`Status::DoneWithCycles`]: fragments that are part of one or more
    ///   cycles.
    /// - [`Status::Done`]: the returned iterator will be empty.
    pub fn punted_iter(&self) -> impl Iterator<Item = FragmentId> + '_ {
        self.punted.keys().copied()
    }
}

impl<P> Solver<P>
where
    P: Problem,
{
    /// Assume the given fragment is already evaluated.
    pub fn assume_evaluated(&mut self, id: FragmentId) -> &mut Self {
        self.mark_solved(id);

        self
    }

    /// Create a clone of `self` that assumes some fragments are already
    /// evaluated.
    ///
    /// This method is useful for trying out assumptions that may need to be
    /// discarted.
    pub fn clone_with_evaluation_assumptions<A>(
        &self,
        assume_evaluated: A,
    ) -> Self
    where
        A: IntoIterator<Item = FragmentId>,
        P: Clone,
    {
        let mut clone = self.clone();
        for id in assume_evaluated {
            clone.assume_evaluated(id);
        }

        clone
    }

    /// Run the solver until all enqueued fragments and their transitive
    /// dependencies are either solved or proven to be part of cycles.
    ///
    /// Returns an interator will all fragments that are part of at least one
    /// cycle. See [`Solver::punted_iter`].
    pub fn run(
        &mut self,
    ) -> Result<impl Iterator<Item = FragmentId> + '_, P::Error> {
        loop {
            match self.step() {
                Ok(false) => return Ok(self.punted_iter()),
                Ok(true) => (),
                Err(err) => return Err(err),
            }
        }
    }

    /// Solve for a single fragment.
    ///
    /// Returns `false` if there are no more fragments that can be evaluated.
    pub fn step(&mut self) -> Result<bool, P::Error> {
        #[cfg(std)]
        let item = self.to_solve.pop();
        #[cfg(not(std))]
        let item = self
            .to_solve
            .iter()
            .next()
            .copied()
            .map(|x| self.to_solve.take(&x).unwrap());

        match item {
            Some(id) => {
                self.dependencies.clear();
                self.problem.direct_dependencies(id, &mut self.dependencies);

                // This is a bit more boilerplatery due to borrowing rules
                {
                    let solved = &self.solved;
                    self.dependencies.retain(|x| !solved.contains(x));
                }

                if self.dependencies.is_empty() {
                    match self.problem.evaluate(id) {
                        Ok(()) => {
                            self.mark_solved(id);

                            Ok(true)
                        }
                        Err(err) => Err(err),
                    }
                } else {
                    self.mark_punted(id);

                    Ok(true)
                }
            }
            None => Ok(false),
        }
    }

    fn mark_solved(&mut self, id: FragmentId) {
        self.solved.insert(id);

        if let Some(dependents) = self.pending_on.remove(&id) {
            for dependent in dependents {
                if *self.punted.get(&dependent).unwrap() == 1 {
                    self.punted.remove(&dependent);
                    self.to_solve.insert(dependent);
                } else {
                    *self.punted.get_mut(&dependent).unwrap() -= 1;
                }
            }
        }
    }

    fn mark_punted(&mut self, id: FragmentId) {
        self.punted.insert(id, self.dependencies.len());

        for dependency in self.dependencies.iter().copied() {
            if !self.solved.contains(&dependency) {
                self.to_solve.insert(dependency);
            }
            self.pending_on.entry(dependency).or_default().push(id);
        }
    }
}

/// Current status of a [`Solver`] instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    /// All fragments have been successfully evaluated.
    Done,

    /// All fragments that could be evaluated have been, but there are still
    /// some that could not due to being part of dependency cycles.
    DoneWithCycles,

    /// There are still fragments that may be evaluated.
    Pending,
}
