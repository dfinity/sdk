use crate::execute::execute::{Execute, SharedExecuteResult};
use crate::execute::promise::Output;
use crate::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ConstNode {
    value: String,
    output: Arc<dyn Output<String>>,
}

#[async_trait]
impl Execute for ConstNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        eprintln!("ConstNode executed with value: {:?}", self.value);

        self.output.set(self.value.clone());
        Ok(())
    }
}

impl ConstNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "const".to_string(),
            inputs: vec![], // no inputs
            outputs: vec!["output".to_string()],
            produces_side_effect: false,
            constructor: Box::new(|config| {
                let value = config.string_param("value");
                let output = config.string_output("output");
                Arc::new(Self { value, output })
            }),
        }
    }
}
