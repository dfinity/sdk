use crate::graph::execute::Execute;
use crate::output_promise::OutputPromise;
use crate::prettify::Prettify;
use crate::registry::node_type::NodeDescriptor;
use std::sync::Arc;

pub struct PrettifyNode {
    input: Arc<OutputPromise<String>>,
    output: Arc<OutputPromise<String>>,
}

#[async_trait::async_trait]
impl Execute for PrettifyNode {
    async fn execute(self: Arc<Self>) {
        let input = self.input.get().await;

        let mut prettify = Prettify::new("target/wasm32-wasip2/release/plugin.wasm").unwrap();
        let prettified = prettify.prettify(&input).unwrap();

        self.output.set(prettified);
    }
}

impl PrettifyNode {
    pub fn descriptor() -> NodeDescriptor {
        NodeDescriptor {
            name: "prettify".to_string(),
            inputs: vec!["input".to_string()],
            outputs: vec!["output".to_string()],
            produces_side_effect: true,
            constructor: |config| {
                let input = config
                    .inputs
                    .get("input")
                    .expect("missing 'input' param")
                    .string()
                    .expect("type mismatch for 'input' output")
                    .clone();
                let output = config
                    .outputs
                    .get("output")
                    .expect("missing 'output' output")
                    .string()
                    .expect("type mismatch for 'output' output");
                Arc::new(Self { input, output })
            },
        }
    }
}
