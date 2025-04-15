use crate::node::Node;
use crate::node_state::NodeEvaluator;
use crate::output_promise::OutputPromise;
use crate::registry::node_type::NodeType;
use crate::registry::node_type_registry::NodeTypeRegistry;
use crate::value::OutputValue;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ConstNode {
    evaluator: NodeEvaluator,
    value: String,
    output: Arc<OutputPromise<String>>,
}

impl ConstNode {
    pub fn new(value: String, output: Arc<OutputPromise<String>>) -> Arc<Self> {
        Arc::new(Self {
            evaluator: NodeEvaluator::new(),
            value,
            output,
        })
    }

    pub fn node_type() -> NodeType {
        NodeType {
            name: "const".to_string(),
            inputs: vec![], // no inputs
            outputs: vec!["output".to_string()],
            constructor: |config| {
                let value = config
                    .params
                    .get("value")
                    .expect("missing 'value' param")
                    .clone();
                let output_promise = config
                    .outputs
                    .get("output")
                    .expect("missing 'value' output")
                    .string()
                    .expect("type mismatch for 'value' output");
                ConstNode::new(value, output_promise)
            },
        }
    }
}

#[async_trait]
impl Node for ConstNode {
    fn produces_side_effect(&self) -> bool {
        false
    }
    fn evaluator(&self) -> &NodeEvaluator {
        &self.evaluator
    }

    async fn evaluate(self: Arc<Self>) {
        println!("ConstNode evaluated with value: {:?}", self.value);
        // just set the value directly, promise will wrap it in a future
        self.output.set(self.value.clone());
    }
}
