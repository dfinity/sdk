use crate::node::Node;
use crate::node_state::NodeState;
use crate::output_promise::OutputPromise;
use crate::value::Value;
use async_trait::async_trait;
use std::sync::Arc;

pub struct ConstNode {
    state: Arc<NodeState>,
    value: Value,
    output: Arc<OutputPromise>,
}

impl ConstNode {
    pub fn new(value: Value) -> Arc<Self> {
        let output = Arc::new(OutputPromise::new());

        let node = Arc::new(Self {
            state: Arc::new(NodeState::new()),
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

    async fn evaluate(self: Arc<Self>) {
        // just set the value directly, promise will wrap it in a future
        self.output.set(self.value.clone());
    }

    async fn ensure_evaluation(self: Arc<Self>) {
        self.state.clone().ensure_evaluation(self.clone()).await;
    }
}
