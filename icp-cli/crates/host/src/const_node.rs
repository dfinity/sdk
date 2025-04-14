use crate::node::Node;
use crate::output_promise::OutputPromise;
use std::sync::Arc;

pub struct ConstNode {
    value: String,
    output: Arc<OutputPromise<String>>,
}

impl ConstNode {
    pub fn new(value: String) -> Arc<Self> {
        Arc::new(Self {
            value,
            output: Arc::new(OutputPromise::new()),
        })
    }

    pub fn output_promise(&self) -> Arc<OutputPromise<String>> {
        Arc::clone(&self.output)
    }
}

#[async_trait::async_trait]
impl Node for ConstNode {
    fn produces_side_effect(&self) -> bool {
        false
    }

    async fn evaluate(&self) {
        self.output.set(self.value.clone()).await;
    }
}
