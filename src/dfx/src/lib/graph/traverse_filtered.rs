// TODO: Somebody, adopt this code (and DFS) to `petgraph`.
use petgraph::{
    data::DataMap,
    visit::{Bfs, IntoNeighborsDirected, VisitMap},
    Direction::Incoming,
};

use crate::lib::error::DfxResult;

pub struct BfsFiltered<NodeId, VM> {
    base: Bfs<NodeId, VM>,
    // node_filter: P,
}

impl<NodeId, VM> BfsFiltered<NodeId, VM> {
    pub fn new(base: Bfs<NodeId, VM>) -> Self {
        Self { base }
    }

    /// TODO: Refactor: Extract `iter` function from here.
    pub fn traverse2<G, P, C, NodeWeight>(
        &mut self,
        graph: G,
        mut predicate: P,
        mut call: C,
    ) -> DfxResult<()>
    where
        C: FnMut(&NodeId, &NodeId) -> DfxResult<()>,
        G: IntoNeighborsDirected<NodeId = NodeId> + DataMap<NodeWeight = NodeWeight>,
        P: FnMut(&NodeId) -> DfxResult<bool>,
        NodeId: Copy + Eq,
        VM: VisitMap<NodeId>,
    {
        while let Some(source_child_id) = &self.base.next(graph) {
            if predicate(source_child_id)? {
                let mut source_parent_iter = graph.neighbors_directed(*source_child_id, Incoming);
                let mut source_parent_id;
                if let Some(id1) = source_parent_iter.next() {
                    source_parent_id = id1;
                    loop {
                        if predicate(&source_parent_id)? {
                            call(&source_parent_id, source_child_id)?;
                            break;
                        }
                        if let Some(id2) = source_parent_iter.next() {
                            source_parent_id = id2;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
