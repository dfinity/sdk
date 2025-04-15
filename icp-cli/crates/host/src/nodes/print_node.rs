use crate::node::Node;
use crate::node_state::NodeEvaluator;
use crate::output_promise::OutputPromise;
use crate::registry::node_type::NodeType;
use async_trait::async_trait;
use std::sync::Arc;

pub struct PrintNode {
    evaluator: NodeEvaluator,
    input: Arc<OutputPromise<String>>,
}

impl PrintNode {
    pub fn node_type() -> NodeType {
        NodeType {
            name: "print".to_string(),
            inputs: vec!["input".to_string()],
            outputs: vec![],
            constructor: |config| {
                let input = config
                    .inputs
                    .get("input")
                    .expect("missing 'input' param")
                    .string()
                    .expect("type mismatch for 'input' output")
                    .clone();
                Arc::new(Self {
                    evaluator: NodeEvaluator::new(),
                    input,
                })
            },
        }
    }
}

#[async_trait]
impl Node for PrintNode {
    fn produces_side_effect(&self) -> bool {
        true
    }
    fn evaluator(&self) -> &NodeEvaluator {
        &self.evaluator
    }

    async fn evaluate(self: Arc<Self>) {
        let value = self.input.get().await;
        println!("PrintNode received: {value}");
    }
}
