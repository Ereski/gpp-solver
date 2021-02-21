use crate::{test::PetgraphProblem, Solver, Status};
use petgraph::Graph;
use std::collections::HashSet;

#[test]
fn should_be_able_to_solve_for_one_fragment_with_no_dependencies() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::Done);
    assert!(punted.is_empty());
    assert_eq!(solver.into_problem().evaluated(), &[p0]);
}

#[test]
fn should_be_able_to_solve_for_two_fragments_with_no_dependencies() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    solver.enqueue_fragment(p1.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::Done);
    assert!(punted.is_empty());
    assert_eq!(
        solver.into_problem().evaluated_set(),
        [p0, p1].iter().copied().collect(),
    );
}

#[test]
fn should_be_able_to_solve_for_one_fragment_with_one_dependency() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::Done);
    assert!(punted.is_empty());
    assert_eq!(solver.into_problem().evaluated(), &[p1, p0]);
}

#[test]
fn should_be_able_to_solve_for_one_fragment_with_two_dependencies() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    let p2 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p0, p2, ());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::Done);
    assert!(punted.is_empty());
    let problem = solver.into_problem();
    let evaluated = problem.evaluated();
    assert!(evaluated == &[p1, p2, p0] || evaluated == &[p2, p1, p0]);
}
