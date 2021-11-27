use crate::{
    reexported::{test, Set},
    test::{PetgraphProblem, CONCURRENCY},
    Solver, Status,
};
use petgraph::Graph;

#[test]
async fn should_be_able_to_solve_for_one_fragment_with_no_dependencies() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::Done);
    assert!(punted.is_empty());
    assert_eq!(solver.into_problem_instance().into_evaluated(), &[p0]);
}

#[test]
async fn should_be_able_to_solve_for_two_fragments_with_no_dependencies() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    solver.enqueue_fragment(p1.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::Done);
    assert!(punted.is_empty());
    assert_eq!(
        solver.into_problem_instance().into_evaluated_set(),
        [p0, p1].into_iter().collect(),
    );
}

#[test]
async fn should_be_able_to_solve_for_one_fragment_with_one_dependency() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::Done);
    assert!(punted.is_empty());
    assert_eq!(solver.into_problem_instance().into_evaluated(), &[p1, p0]);
}

#[test]
async fn should_be_able_to_solve_for_one_fragment_with_two_dependencies() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    let p2 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p0, p2, ());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::Done);
    assert!(punted.is_empty());
    let problem = solver.into_problem_instance();
    let evaluated = problem.into_evaluated();
    assert!(evaluated == &[p1, p2, p0] || evaluated == &[p2, p1, p0]);
}
