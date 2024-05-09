// TODO: Integrate it into `petgraph` library.

use std::collections::HashMap;
use std::hash::Hash;

use petgraph::{csr::IndexType, graph::{DefaultIx, NodeIndex}, Directed, EdgeType, Graph};

struct GraphWithNodesMap<N, E, Ty = Directed, Ix = DefaultIx> {
    graph: Graph<N, E, Ty, Ix>,
    nodes: HashMap<N, NodeIndex<Ix>>,
}

impl<N, E, Ty, Ix> GraphWithNodesMap<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    pub fn update_node(&mut self, weight: &N) -> NodeIndex<Ix>
        where N: Eq + Hash + Clone,
    {
        // TODO: Get rid of two `clone`s.
        *self.nodes
            .entry(weight.clone())
            .or_insert_with(|| self.graph.add_node(weight.clone()))
    }
}

impl<N, E> GraphWithNodesMap<N, E, Directed> {
    pub fn new() -> Self
    {
        Self {
            graph: Graph::new(),
            nodes: HashMap::new(),
        }
    }
}

