use crate::workflow::execute::execute::{Execute, SharedExecuteResult};
use crate::workflow::execute::promise::{InputRef, Output, OutputRef};
use crate::workflow::registry::edge::EdgeType;
use crate::workflow::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ConstStringNode {
    value: InputRef<String>,
    output: OutputRef<String>,
}

#[async_trait]
impl Execute for ConstStringNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        let value = self.value.get().await?;
        eprintln!("ConstNode executed with value: {:?}", value);

        self.output.set(value.clone());
        Ok(())
    }
}

impl ConstStringNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "const-string".to_string(),
            inputs: HashMap::from([("value".to_string(), EdgeType::String)]),
            outputs: HashMap::from([("output".to_string(), EdgeType::String)]),
            produces_side_effect: false,
            constructor: Box::new(|config| {
                let value = config.string_input("value")?;
                let output = config.string_output("output");
                Ok(Arc::new(Self { value, output }))
            }),
        }
    }
}
