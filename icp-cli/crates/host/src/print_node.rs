use crate::node::Node;
use crate::output_promise::OutputPromise;
use std::sync::Arc;

pub struct PrintNode {
    value: Arc<OutputPromise<String>>,
}

impl PrintNode {
    pub fn new(value: Arc<OutputPromise<String>>) -> Arc<Self> {
        Arc::new(Self { value })
    }
}

#[async_trait::async_trait]
impl Node for PrintNode {
    fn produces_side_effect(&self) -> bool {
        true
    }

    async fn evaluate(&self) {
        let val = self.value.get().await;
        println!("PrintNode: {}", val);
    }
}
