// TODO: Somebody, adopt this code to `pethgraph`.
use petgraph::{data::DataMap, visit::{Dfs, IntoNeighbors, VisitMap}};

pub struct DfsFiltered<NodeId, VM>
    // where P: FnMut(&N) -> bool
{
    base: Dfs<NodeId, VM>,
    // node_filter: P,
}

impl<NodeId, VM> DfsFiltered<NodeId, VM> {
    pub fn new(base: Dfs<NodeId, VM>) -> Self {
        Self {
            base
        }
    }

    pub fn traverse<G, P, C, NodeWeight>(&mut self, graph: G, predicate: P, call: C)
    where C: Fn(&NodeId, &NodeId) -> (),
          G: IntoNeighbors<NodeId = NodeId> + DataMap<NodeWeight = NodeWeight>,
          P: Fn(&NodeId) -> bool,
          NodeId: Copy + PartialEq,
          VM: VisitMap<NodeId>,
    {
        while let Some(item) = &self.base.next(graph) {
            if predicate(item) {
                let parent = self.base.stack.iter().rev().find(predicate);
                if let Some(parent) = &parent {
                    call(parent, item);
                }
            }
        }
    }
}