// TODO: Somebody, adopt this code (and DFS) to `petgraph`.
use petgraph::{
    data::DataMap,
    visit::{Bfs, IntoNeighborsDirected, VisitMap},
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
        while let Some(child_id) = &self.base.next(graph) {
            if predicate(&child_id)? {
                let mut parent_iter = self.base.stack.iter().rev();
                let parent_id =
                    parent_iter
                        .find_map(|&id| -> Option<DfxResult<NodeId>> {
                            match predicate(&id) {
                                Ok(true) => Some(Ok(id)),
                                Ok(false) => None,
                                Err(err) => Some(Err(err)),
                            }
                        })
                        .transpose()?;
                if let Some(parent_id) = parent_id {
                    assert!(parent_id != *child_id);
                    call(&parent_id, child_id)?;
                }
            }
        }
        Ok(())
    }
}
