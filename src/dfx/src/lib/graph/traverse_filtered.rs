// TODO: Somebody, adopt this code to `pethgraph`.
use petgraph::{data::DataMap, visit::{Dfs, IntoNeighbors, VisitMap}};

pub struct DfsFiltered<N, VM>
    // where P: FnMut(&N) -> bool
{
    base: Dfs<N, VM>,
    // node_filter: P,
}

impl<N, VM> DfsFiltered<N, VM> {
    pub fn new(base: Dfs<N, VM>) -> Self {
        Self {
            base
        }
    }

    pub fn traverse<G, P, C>(&mut self, graph: G, predicate: P, call: C)
    where C: Fn(&N, &N) -> (),
          G: IntoNeighbors<NodeId = N> + DataMap<NodeWeight = N>,
          P: Fn(&N) -> bool,
          N: Copy + PartialEq,
          VM: VisitMap<N>,
    {
        while let Some(item) = &self.base.next(graph) {
            if predicate(item) {
                let parent = self.base.stack.iter().rev().find(
                    |&entry| if let Some(elt) = graph.node_weight(*entry) {
                        predicate(elt)
                    } else {
                        false
                    }
                );
                if let Some(parent) = parent {
                    call(parent, item);
                }
            }
        }
    }
}