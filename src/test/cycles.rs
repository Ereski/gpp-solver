use crate::{test::PetgraphProblem, FragmentId, Solver, Status};
use petgraph::{graph::NodeIndex, Graph};
use std::collections::HashSet;

#[test]
fn should_be_able_to_punt_a_self_cycle() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p0, ());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0]));
    assert_eq!(solver.into_problem().evaluated(), &[]);
}

#[test]
fn should_be_able_to_punt_a_two_node_cycle() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p1, p0, ());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    solver.enqueue_fragment(p1.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0, p1]));
    assert_eq!(solver.into_problem().evaluated(), &[]);
}

#[test]
fn should_be_able_to_punt_a_self_cycle_within_a_two_node_cycle() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p1, p0, ());
    dependency_graph.add_edge(p0, p0, ());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    solver.enqueue_fragment(p1.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0, p1]));
    assert_eq!(solver.into_problem().evaluated(), &[]);
}

#[test]
fn should_be_able_to_punt_a_two_intersecting_cycles() {
    let mut dependency_graph = Graph::new();
    let p0 = dependency_graph.add_node(());
    let p1 = dependency_graph.add_node(());
    let p2 = dependency_graph.add_node(());
    dependency_graph.add_edge(p0, p1, ());
    dependency_graph.add_edge(p1, p0, ());
    dependency_graph.add_edge(p1, p2, ());
    dependency_graph.add_edge(p2, p1, ());

    let mut solver = Solver::new(PetgraphProblem::new(dependency_graph));
    solver.enqueue_fragment(p0.index().into());
    let punted = solver.run().unwrap().collect::<HashSet<_>>();

    assert_eq!(solver.status(), Status::DoneWithCycles);
    assert_eq!(punted, index_slice_as_set(&[p0, p1, p2]));
    assert_eq!(solver.into_problem().evaluated(), &[]);
}

fn index_slice_as_set(indexes: &[NodeIndex<u32>]) -> HashSet<FragmentId> {
    indexes.iter().map(|x| x.index().into()).collect()
}
