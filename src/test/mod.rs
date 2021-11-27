use crate::{
    reexported::{Box, Mutex, NonZeroUsize, Set, Vec},
    {FragmentId, Problem},
};
use async_trait::async_trait;
use petgraph::{graph::NodeIndex, visit::EdgeRef, Directed, Graph};
use void::Void;

mod cycles;
mod sanity;
mod tree;

const CONCURRENCY: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(2) };

struct PetgraphProblem {
    dependency_graph: Graph<(), (), Directed>,
    evaluation_order: Mutex<Vec<NodeIndex<u32>>>,
}

impl PetgraphProblem {
    fn new(dependency_graph: Graph<(), (), Directed>) -> Self {
        Self {
            dependency_graph,
            evaluation_order: Mutex::new(Vec::new()),
        }
    }

    fn into_evaluated(self) -> Vec<NodeIndex<u32>> {
        self.evaluation_order.into_inner()
    }

    fn into_evaluated_set(self) -> Set<NodeIndex<u32>> {
        self.evaluation_order.into_inner().into_iter().collect()
    }
}

#[async_trait]
impl Problem for PetgraphProblem {
    type Error = Void;

    async fn direct_dependencies(
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

    async fn evaluate(&self, id: FragmentId) -> Result<(), Self::Error> {
        self.evaluation_order
            .lock()
            .await
            .push(NodeIndex::new(id.into()));

        Ok(())
    }
}
