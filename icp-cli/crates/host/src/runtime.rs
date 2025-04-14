use crate::node::Node;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{Notify, RwLock};
use tokio::task::JoinHandle;

// Represents the result of evaluating a node
#[derive(Debug, Clone)]
pub enum OutputValue {
    String(String),
    // Add other types as needed
}

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

    pub async fn evaluate_all(&self) {
        let mut handles: Vec<JoinHandle<()>> = vec![];

        for node in &self.nodes {
            let node_clone = Arc::clone(node);
            handles.push(tokio::spawn(async move {
                node_clone.evaluate().await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
