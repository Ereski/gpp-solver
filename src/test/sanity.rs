use crate::{
    reexported::{test, Box, Vec},
    test::CONCURRENCY,
    FragmentId, Problem, Solver, Status,
};
use async_trait::async_trait;
use void::Void;

struct PanicProblem;

#[async_trait]
impl Problem for PanicProblem {
    type Error = Void;

    async fn direct_dependencies(
        &self,
        _: FragmentId,
        _: &mut Vec<FragmentId>,
    ) {
        unreachable!()
    }

    async fn evaluate(&self, _: FragmentId) -> Result<(), Self::Error> {
        unreachable!()
    }
}

#[test]
async fn empty_solver_status_should_be_done() {
    assert_eq!(Solver::new(()).status().await, Status::Done);
}

#[test]
async fn empty_solver_punted_iter_must_be_empty() {
    assert_eq!(Solver::new(()).punted_iter().await.first(), None);
}

#[test]
async fn non_empty_unexecuted_solver_status_should_be_pending() {
    assert_eq!(
        Solver::new(())
            .enqueue_fragment(0.into())
            .await
            .status()
            .await,
        Status::Pending,
    );
}

#[test]
async fn non_empty_unexecuted_solver_punted_iter_must_be_empty() {
    assert_eq!(
        Solver::new(())
            .enqueue_fragment(0.into())
            .await
            .punted_iter()
            .await
            .first(),
        None,
    );
}

#[test]
async fn stepping_an_empty_solver_must_return_ok_false() {
    assert_eq!(Solver::new(PanicProblem).step().await, Ok(false));
}

#[test]
async fn running_an_empty_solver_must_return_ok_wth_an_empty_iterator() {
    assert_eq!(
        Solver::new(PanicProblem)
            .run(CONCURRENCY)
            .await
            .unwrap()
            .first(),
        None
    );
}
