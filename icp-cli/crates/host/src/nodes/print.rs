use crate::graph::execute::Execute;
use crate::output_promise::OutputPromise;
use crate::registry::node_type::NodeDescriptor;
use async_trait::async_trait;
use std::sync::Arc;

pub struct PrintNode {
    input: Arc<OutputPromise<String>>,
}

#[async_trait]
impl Execute for PrintNode {
    async fn execute(self: Arc<Self>) {
        let value = self.input.get().await;
        println!("PrintNode received: {value}");
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
                let input = config
                    .inputs
                    .get("input")
                    .expect("missing 'input' param")
                    .string()
                    .expect("type mismatch for 'input' output")
                    .clone();
                Arc::new(Self { input })
            },
        }
    }
}
