use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{Notify, RwLock};

// Represents the result of evaluating a node
#[derive(Debug, Clone)]
pub enum OutputValue {
    String(String),
    // Add other types as needed
}

// Basic promise that fulfills when the node is evaluated
pub struct OutputPromise {
    notify: Arc<Notify>,
    result: Arc<RwLock<Option<OutputValue>>>,
}

impl OutputPromise {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            result: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get(&self) -> OutputValue {
        loop {
            if let Some(value) = self.result.read().await.clone() {
                return value;
            }
            self.notify.notified().await;
        }
    }

    pub async fn fulfill(&self, value: OutputValue) {
        let mut result = self.result.write().await;
        *result = Some(value);
        self.notify.notify_waiters();
    }
}

#[async_trait]
pub trait Node: Send + Sync {
    fn id(&self) -> &str;

    // The runtime will call this to start evaluating the node.
    async fn evaluate(&self, runtime: &GraphRuntime);

    // Expose a promise for a given output
    fn get_output(&self, name: &str) -> Option<Arc<OutputPromise>>;
}

pub struct GraphRuntime {
    nodes: Mutex<HashMap<String, Arc<dyn Node>>>,
}

impl GraphRuntime {
    pub fn new() -> Self {
        Self {
            nodes: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_node(&self, node: Arc<dyn Node>) {
        let id = node.id().to_string();
        self.nodes.lock().unwrap().insert(id, node);
    }

    pub async fn get_output(&self, node_id: &str, output_name: &str) -> Option<Arc<OutputPromise>> {
        let nodes = self.nodes.lock().unwrap();
        nodes
            .get(node_id)
            .and_then(|node| node.get_output(output_name))
    }

    pub async fn evaluate_node(&self, node_id: &str) {
        if let Some(node) = self.nodes.lock().unwrap().get(node_id) {
            node.evaluate(self).await;
        }
    }
}
