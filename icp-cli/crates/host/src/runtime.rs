use crate::node::Node;
use std::sync::Arc;

pub struct Runtime {
    nodes: Vec<Arc<dyn Node>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn add_node(&mut self, node: Arc<dyn Node>) {
        self.nodes.push(node);
    }

    // pub async fn run_graph(&self) {
    //     let futures = self
    //         .nodes
    //         .iter()
    //         .filter(|n| n.produces_side_effect())
    //         .map(|n| n.clone().ensure_evaluation())
    //         .collect::<Vec<_>>();
    //
    //     futures::future::join_all(futures).await;
    // }
}
