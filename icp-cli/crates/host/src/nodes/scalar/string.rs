use crate::execute::execute::{Execute, SharedExecuteResult};
use crate::execute::promise::{Output, OutputRef};
use crate::registry::edge::EdgeType;
use crate::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ConstStringNode {
    value: String,
    output: OutputRef<String>,
}

#[async_trait]
impl Execute for ConstStringNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        eprintln!("ConstNode executed with value: {:?}", self.value);

        self.output.set(self.value.clone());
        Ok(())
    }
}

impl ConstStringNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "const-string".to_string(),
            inputs: HashMap::new(), // no inputs
            outputs: HashMap::from([("output".to_string(), EdgeType::String)]),
            produces_side_effect: false,
            constructor: Box::new(|config| {
                let value = config.string_param("value");
                let output = config.string_output("output");
                Ok(Arc::new(Self { value, output }))
            }),
        }
    }
}
