use std::iter::once;

use petgraph::{data::DataMap, visit::{Bfs, IntoNeighbors, VisitMap}};

use crate::lib::error::DfxResult;

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

    #[allow(unused)]
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
                    // Requested to document the next line behavior in https://github.com/petgraph/petgraph/issues/634
                    let source_parent_id = self.base.stack.iter().map(|e| *e).chain(once(first_id)).find(&mut predicate);
                    if let Some(source_parent_id) = &source_parent_id {
                        (&mut call)(source_parent_id, &source_child_id);
                    }
                }
            }
        }
    }

    pub fn traverse2<G, P, C, NodeWeight>(&mut self, graph: G, mut predicate: P, mut call: C) -> DfxResult<()>
    where C: FnMut(&NodeId, &NodeId) -> DfxResult<()>,
          G: IntoNeighbors<NodeId = NodeId> + DataMap<NodeWeight = NodeWeight>,
          P: FnMut(&NodeId) -> DfxResult<bool>,
          NodeId: Copy + Eq,
          VM: VisitMap<NodeId>,
    {
        if let Some(first_id) = self.base.next(graph) {
            while let Some(source_child_id) = &self.base.next(graph) {
                if (&mut predicate)(source_child_id)? {
                    // Requested to document the next line behavior in https://github.com/petgraph/petgraph/issues/634
                    let source_parent_id = self.base.stack.iter().map(|e| *e).chain(once(first_id))
                        .filter_map(|x| (&mut predicate)(&x)
                            .map_or_else(|e| Some(Err(e)), |v| if v { Some(Ok(x)) } else { None }))
                        .next().transpose()?;
                    if let Some(source_parent_id) = &source_parent_id {
                        (&mut call)(source_parent_id, &source_child_id)?;
                    }
                }
            }
        }
        Ok(())
    }
}