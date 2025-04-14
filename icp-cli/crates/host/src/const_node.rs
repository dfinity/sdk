use crate::runtime::{GraphRuntime, Node, OutputPromise, OutputValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct ConstNode {
    id: String,
    value: String,
    output_name: String,
    output: Arc<OutputPromise>,
    evaluated: OnceCell<()>,
}

impl ConstNode {
    pub fn new(
        id: impl Into<String>,
        output_name: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            output_name: output_name.into(),
            output: Arc::new(OutputPromise::new()),
            evaluated: OnceCell::new(),
        }
    }
}

#[async_trait::async_trait]
impl Node for ConstNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn get_output(&self, name: &str) -> Option<Arc<OutputPromise>> {
        if name == self.output_name {
            Some(self.output.clone())
        } else {
            None
        }
    }

    async fn evaluate(&self, _runtime: &GraphRuntime) {
        // Only evaluate once
        if self.evaluated.set(()).is_ok() {
            // Simulate work (optional)
            // tokio::time::sleep(Duration::from_millis(50)).await;
            self.output
                .fulfill(OutputValue::String(self.value.clone()))
                .await;
        }
    }
}
