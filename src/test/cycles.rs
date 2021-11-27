use crate::{
    reexported::{test, Set},
    test::{PetgraphProblem, CONCURRENCY},
    FragmentId, Solver, Status,
};
use petgraph::{graph::NodeIndex, Graph};

#[test]
async fn should_be_able_to_punt_a_self_cycle() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p0, ());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0]));
    assert_eq!(solver.into_problem_instance().into_evaluated(), &[]);
}

#[test]
async fn should_be_able_to_punt_a_two_node_cycle() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p1, p0, ());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    solver.enqueue_fragment(p1.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0, p1]));
    assert_eq!(solver.into_problem_instance().into_evaluated(), &[]);
}

#[test]
async fn should_be_able_to_punt_a_self_cycle_within_a_two_node_cycle() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p1, p0, ());
    dependency_graph.add_edge(p0, p0, ());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    solver.enqueue_fragment(p1.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0, p1]));
    assert_eq!(solver.into_problem_instance().into_evaluated(), &[]);
}

#[test]
async fn should_be_able_to_punt_a_two_intersecting_cycles() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    let p2 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p1, p0, ());
    dependency_graph.add_edge(p1, p2, ());
    dependency_graph.add_edge(p2, p1, ());

    let solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into()).await;
    let punted = solver
        .run(CONCURRENCY)
        .await
        .unwrap()
        .into_iter()
        .collect::<Set<_>>();

    assert_eq!(solver.status().await, Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0, p1, p2]));
    assert_eq!(solver.into_problem_instance().into_evaluated(), &[]);
}

fn index_slice_as_set(indexes: &[NodeIndex<u32>]) -> Set<FragmentId> {
    indexes.iter().map(|x| x.index().into()).collect()
}
