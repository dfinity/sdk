// TODO: Somebody, adopt this code (and DFS) to `petgraph`.
use petgraph::{data::DataMap, visit::IntoNeighborsDirected};

use crate::lib::error::DfxResult;

pub struct DfsFiltered {}

// FIXME: This is DFS, not BFS.
impl DfsFiltered {
    pub fn new() -> Self {
        Self {}
    }

    /// TODO: Refactor: Extract `iter` function from here.
    pub fn traverse2<G, NodeId, P, C, NodeWeight>(
        &mut self,
        graph: G,
        mut predicate: P,
        mut call: C,
        node_id: NodeId,
    ) -> DfxResult<()>
    where
        C: FnMut(&NodeId, &NodeId) -> DfxResult<()>,
        G: IntoNeighborsDirected<NodeId = NodeId> + DataMap<NodeWeight = NodeWeight>,
        NodeId: Copy + Eq,
        P: FnMut(&NodeId) -> DfxResult<bool>,
    {
        Self::traverse2_recursive(graph, &mut predicate, &mut call, node_id, &mut Vec::new())
    }

    fn traverse2_recursive<G, NodeId, P, C, NodeWeight>(
        graph: G,
        predicate: &mut P,
        call: &mut C,
        node_id: NodeId,
        ancestors: &mut Vec<NodeId>,
    ) -> DfxResult<()>
    where
        C: FnMut(&NodeId, &NodeId) -> DfxResult<()>,
        G: IntoNeighborsDirected<NodeId = NodeId> + DataMap<NodeWeight = NodeWeight>,
        NodeId: Copy + Eq,
        P: FnMut(&NodeId) -> DfxResult<bool>,
        NodeId: Copy + Eq,
    {
        if !predicate(&node_id)? {
            return Ok(());
        }
        let ancestor_id = ancestors
            .iter()
            .rev()
            .find_map(|&id| -> Option<DfxResult<NodeId>> {
                match predicate(&id) {
                    Ok(true) => Some(Ok(id)),
                    Ok(false) => None,
                    Err(err) => Some(Err(err)),
                }
            })
            .transpose()?;
        if let Some(ancestor_id) = ancestor_id {
            assert!(ancestor_id != node_id);
            call(&ancestor_id, &node_id)?;
        }
        ancestors.push(node_id);
        for subnode_id in graph.neighbors(node_id) {
            Self::traverse2_recursive(graph, predicate, call, subnode_id, ancestors)?;
        }
        ancestors.pop();

        Ok(())
    }
}
