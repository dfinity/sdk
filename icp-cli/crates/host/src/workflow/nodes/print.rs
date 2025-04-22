use crate::workflow::execute::execute::{Execute, SharedExecuteResult};
use crate::workflow::execute::promise::{Input, InputRef};
use crate::workflow::registry::edge::EdgeType;
use crate::workflow::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::collections::HashMap;
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
            inputs: HashMap::from([("input".to_string(), EdgeType::String)]),
            outputs: HashMap::new(),
            produces_side_effect: true,
            constructor: Box::new(|config| {
                let input = config.string_source("input")?;
                Ok(Arc::new(Self { input }))
            }),
        }
    }
}
