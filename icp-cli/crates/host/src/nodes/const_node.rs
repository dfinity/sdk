use crate::node::Node;
use crate::node_state::NodeEvaluator;
use crate::output_promise::OutputPromise;
use crate::value::OutputValue;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ConstNode {
    evaluator: NodeEvaluator,
    value: OutputValue,
    output: Arc<OutputPromise>,
}

impl ConstNode {
    pub fn new(value: OutputValue) -> Arc<Self> {
        let output = Arc::new(OutputPromise::new());

        let node = Arc::new(Self {
            evaluator: NodeEvaluator::new(),
            value,
            output,
        });

        // Now set up the owner safely
        let weak_self = Arc::downgrade(&(node.clone() as Arc<dyn Node>));
        node.output.set_owner(weak_self);

        node
    }
    pub fn output_promise(self: &Arc<Self>) -> Arc<OutputPromise> {
        self.output.clone()
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
