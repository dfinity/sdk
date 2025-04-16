use crate::execution::execute::{Execute, SharedExecuteResult};
use crate::execution::promise::Promise;
use crate::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::sync::Arc;

pub struct PrintNode {
    input: Arc<Promise<String>>,
}

#[async_trait]
impl Execute for PrintNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        let value = self.input.get().await?;
        println!("PrintNode received: {value}");
        Ok(())
    }
}

impl PrintNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "print".to_string(),
            inputs: vec!["input".to_string()],
            outputs: vec![],
            produces_side_effect: true,
            constructor: |config| {
                let input = config.string_input("input");
                Arc::new(Self { input })
            },
        }
    }
}
