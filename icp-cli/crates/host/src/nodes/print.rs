use crate::execute::execute::{Execute, SharedExecuteResult};
use crate::execute::promise::{Input, InputRef};
use crate::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::sync::Arc;

pub struct PrintNode {
    input: InputRef<String>,
}

#[async_trait]
impl Execute for PrintNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        eprintln!("PrintNode executing");
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
            constructor: Box::new(|config| {
                let input = config.string_source("input");
                Arc::new(Self { input })
            }),
        }
    }
}
