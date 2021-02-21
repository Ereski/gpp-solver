use crate::{FragmentId, Problem};
use petgraph::{graph::NodeIndex, visit::EdgeRef, Directed, Graph};
use std::collections::HashSet;
use void::Void;

mod cycles;
mod sanity;
mod tree;

struct PetgraphProblem {
    dependency_graph: Graph<(), (), Directed>,
    evaluation_order: Vec<NodeIndex<u32>>,
}

impl PetgraphProblem {
    fn new(dependency_graph: Graph<(), (), Directed>) -> Self {
        Self {
            dependency_graph,
            evaluation_order: Vec::new(),
        }
    }

    fn evaluated(&self) -> &[NodeIndex<u32>] {
        &self.evaluation_order
    }

    fn evaluated_set(&self) -> HashSet<NodeIndex<u32>> {
        self.evaluation_order.iter().copied().collect()
    }
}

impl Problem for PetgraphProblem {
    type Error = Void;

    fn direct_dependencies(
        &self,
        id: FragmentId,
        dependecies: &mut Vec<FragmentId>,
    ) {
        dependecies.extend(
            self.dependency_graph
                .edges(NodeIndex::new(id.into()))
                .map(|x| FragmentId::from(x.target().index())),
        )
    }

    fn evaluate(&mut self, id: FragmentId) -> Result<(), Self::Error> {
        self.evaluation_order.push(NodeIndex::new(id.into()));

        Ok(())
    }
}
