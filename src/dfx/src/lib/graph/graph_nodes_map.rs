// TODO: Integrate it into `petgraph` library.

use std::collections::HashMap;
use std::hash::Hash;

use petgraph::{
    graph::EdgeIndex,
    graph::IndexType,
    graph::{DefaultIx, NodeIndex},
    Directed, EdgeType, Graph,
};

pub struct GraphWithNodesMap<N, E, Ty = Directed, Ix = DefaultIx> {
    graph: Graph<N, E, Ty, Ix>,
    nodes: HashMap<N, NodeIndex<Ix>>,
}

impl<N, E, Ty, Ix> GraphWithNodesMap<N, E, Ty, Ix> {
    pub fn graph(&self) -> &Graph<N, E, Ty, Ix> {
        &self.graph
    }
    pub fn nodes(&self) -> &HashMap<N, NodeIndex<Ix>> {
        &self.nodes
    }
}

impl<N, E, Ty, Ix> GraphWithNodesMap<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    pub fn update_node(&mut self, weight: &N) -> NodeIndex<Ix>
    where
        N: Eq + Hash + Clone,
    {
        // TODO: Get rid of two `clone`s (apparently, requires data stucture change).
        *self
            .nodes
            .entry(weight.clone())
            .or_insert_with(|| self.graph.add_node(weight.clone()))
    }
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        self.graph.update_edge(a, b, weight)
    }
}

impl<N, E> GraphWithNodesMap<N, E, Directed> {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            nodes: HashMap::new(),
        }
    }
}
