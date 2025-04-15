use crate::node::Node;
use crate::node_state::NodeEvaluator;
use crate::output_promise::OutputPromise;
use crate::value::OutputValue;
use async_trait::async_trait;
use std::sync::Arc;

pub struct PrintNode {
    evaluator: NodeEvaluator,
    input: Arc<OutputPromise>,
}

impl PrintNode {
    pub fn new(input: Arc<OutputPromise>) -> Arc<Self> {
        Arc::new(Self {
            evaluator: NodeEvaluator::new(),
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
        if let OutputValue::String(s) = value {
            println!("PrintNode received: {}", s);
        }
    }

    async fn ensure_evaluation(self: Arc<Self>) {
        self.evaluator.ensure_evaluation(self.clone()).await;
    }
}
