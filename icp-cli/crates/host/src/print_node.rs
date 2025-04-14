use crate::output_promise::{Node, OutputPromise};
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
    async fn evaluate(&self) {
        let val = self.value.get().await;
        println!("PrintNode: {}", val);
    }
}
