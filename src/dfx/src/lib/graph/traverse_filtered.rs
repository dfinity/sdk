use std::iter::once;

// TODO: Somebody, adopt this code to `petgraph`.
use petgraph::{data::DataMap, visit::{Bfs, Dfs, IntoNeighbors, VisitMap}};

#[allow(unused)]
pub struct DfsFiltered<NodeId, VM> {
    base: Dfs<NodeId, VM>,
    // node_filter: P,
}

impl<NodeId, VM> DfsFiltered<NodeId, VM> {
    #[allow(unused)]
    pub fn new(base: Dfs<NodeId, VM>) -> Self {
        Self {
            base
        }
    }

    #[allow(unused)]
    pub fn traverse<G, P, C, NodeWeight>(&mut self, graph: G, mut predicate: P, mut call: C)
    where C: FnMut(&NodeId, &NodeId) -> (),
          G: IntoNeighbors<NodeId = NodeId> + DataMap<NodeWeight = NodeWeight>,
          P: FnMut(&NodeId) -> bool,
          NodeId: Copy + Eq,
          VM: VisitMap<NodeId>,
    {
        while let Some(source_item_id) = &self.base.next(graph) {
            if (&mut predicate)(source_item_id) {
                let source_parent_id = self.base.stack.iter().map(|e| *e).rev().find(&mut predicate); // FIXME: `rev()` here?
                if let Some(source_parent_id) = &source_parent_id {
                    (&mut call)(source_parent_id, &source_item_id);
                }
            }
        }
    }
}

pub struct BfsFiltered<NodeId, VM> {
    base: Bfs<NodeId, VM>,
    // node_filter: P,
}

impl<NodeId, VM> BfsFiltered<NodeId, VM> {
    pub fn new(base: Bfs<NodeId, VM>) -> Self {
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
        if let Some(first_id) = self.base.next(graph) {
            while let Some(source_child_id) = &self.base.next(graph) {
                if (&mut predicate)(source_child_id) {
                    // TODO: Create a `petgraph` issue asking to explain the next line of code workings.
                    let source_parent_id = self.base.stack.iter().map(|e| *e).chain(once(first_id)).find(&mut predicate);
                    if let Some(source_parent_id) = &source_parent_id {
                        (&mut call)(source_parent_id, &source_child_id);
                    }
                }
            }
        }
    }
}