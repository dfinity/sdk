use crate::graph::execute::{Execute, SharedExecuteResult};
use crate::output_promise::OutputPromise;
use crate::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ConstNode {
    value: String,
    output: Arc<OutputPromise<String>>,
}

#[async_trait]
impl Execute for ConstNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        println!("ConstNode executed with value: {:?}", self.value);
        // just set the value directly, promise will wrap it in a future
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
            constructor: |config| {
                let value = config
                    .params
                    .get("value")
                    .expect("missing 'value' param")
                    .clone();
                let output = config
                    .outputs
                    .get("output")
                    .expect("missing 'value' output")
                    .string()
                    .expect("type mismatch for 'value' output");
                Arc::new(Self { value, output })
            },
        }
    }
}
