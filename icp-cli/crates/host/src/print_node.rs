use crate::node::Node;
use crate::node_state::NodeState;
use crate::output_promise::OutputPromise;
use crate::value::Value;
use async_trait::async_trait;
use futures_util::future::{BoxFuture, Shared};
use std::cell::OnceCell;
use std::sync::Arc;

pub struct PrintNode {
    state: Arc<NodeState>,
    input: Arc<OutputPromise>,
}

impl PrintNode {
    pub fn new(input: Arc<OutputPromise>) -> Arc<Self> {
        Arc::new(Self {
            state: Arc::new(NodeState::new()),
            input,
        })
    }
}

#[async_trait]
impl Node for PrintNode {
    fn produces_side_effect(&self) -> bool {
        true
    }

    async fn evaluate(self: Arc<Self>) {
        let value = self.input.get().await;
        if let Value::String(s) = value {
            println!("PrintNode received: {}", s);
        }
    }

    async fn ensure_evaluation(self: Arc<Self>) {
        self.state.clone().ensure_evaluation(self.clone()).await;
    }

    // fn evaluation_cell(&self) -> &OnceCell<Shared<BoxFuture<'static, ()>>> {
    //     &self.eval_cell
    // }
}
