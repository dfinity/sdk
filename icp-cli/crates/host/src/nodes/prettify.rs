use crate::execute::execute::{Execute, SharedExecuteResult};
use crate::execute::promise::{Input, InputRef, Output, OutputRef};
use crate::execute::GraphExecutionError;
use crate::prettify::Prettify;
use crate::registry::edge::EdgeType;
use crate::registry::node_type::NodeDescriptor;
use std::collections::HashMap;
use std::sync::Arc;

pub struct PrettifyNode {
    input: InputRef<String>,
    output: OutputRef<String>,
}

#[async_trait::async_trait]
impl Execute for PrettifyNode {
    async fn execute(self: Arc<Self>) -> SharedExecuteResult {
        let input = self.input.get().await?;

        let mut prettify = Prettify::new("target/wasm32-wasip2/release/plugin.wasm")
            .map_err(GraphExecutionError::PrettifyError)?;
        let prettified = prettify
            .prettify(&input)
            .map_err(GraphExecutionError::PrettifyError)?;

        self.output.set(prettified);
        Ok(())
    }
}

impl PrettifyNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "prettify".to_string(),
            inputs: HashMap::from([("input".to_string(), EdgeType::String)]),
            outputs: HashMap::from([("output".to_string(), EdgeType::String)]),
            produces_side_effect: true,
            constructor: Box::new(|config| {
                let input = config.string_source("input")?;
                let output = config.string_output("output");
                Ok(Arc::new(Self { input, output }))
            }),
        }
    }
}
