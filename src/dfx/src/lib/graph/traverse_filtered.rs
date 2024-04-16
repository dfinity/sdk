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

    pub fn traverse<G, P, C, NodeWeight>(&mut self, graph: G, mut predicate: P, mut call: C)
    where C: FnMut(&NodeId, &NodeId) -> (),
          G: IntoNeighbors<NodeId = NodeId> + DataMap<NodeWeight = NodeWeight>,
          P: FnMut(&NodeId) -> bool,
          NodeId: Copy + Eq,
          VM: VisitMap<NodeId>,
    {
        while let Some(source_item_id) = &self.base.next(graph) {
            if (&mut predicate)(source_item_id) {
                let source_parent_id = self.base.stack.iter().map(|e| *e).rev().find(&mut predicate);
                if let Some(source_parent_id) = &source_parent_id {
                    (&mut call)(source_parent_id, &source_item_id);
                }
            }
        }
    }
}