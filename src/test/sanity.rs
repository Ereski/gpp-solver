use crate::{FragmentId, Problem, Solver, Status};
use void::Void;

struct PanicProblem;

impl Problem for PanicProblem {
    type Error = Void;

    fn direct_dependencies(&self, _: FragmentId, _: &mut Vec<FragmentId>) {
        unreachable!()
    }

    fn evaluate(&mut self, _: FragmentId) -> Result<(), Self::Error> {
        unreachable!()
    }
}

#[test]
fn empty_solver_status_should_be_done() {
    assert_eq!(Solver::new(()).status(), Status::Done);
}

#[test]
fn empty_solver_punted_iter_must_be_empty() {
    assert_eq!(Solver::new(()).punted_iter().next(), None);
}

#[test]
fn non_empty_unexecuted_solver_status_should_be_pending() {
    assert_eq!(
        Solver::new(()).enqueue_fragment(0.into()).status(),
        Status::Pending,
    );
}

#[test]
fn non_empty_unexecuted_solver_punted_iter_must_be_empty() {
    assert_eq!(
        Solver::new(())
            .enqueue_fragment(0.into())
            .punted_iter()
            .next(),
        None,
    );
}

#[test]
fn stepping_an_empty_solver_must_return_ok_false() {
    assert_eq!(Solver::new(PanicProblem).step(), Ok(false));
}

#[test]
fn running_an_empty_solver_must_return_ok_wth_an_empty_iterator() {
    assert_eq!(Solver::new(PanicProblem).run().unwrap().next(), None);
}
