use crate::node_state::NodeEvaluator;
use crate::output_promise::OutputPromise;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait Node: Send + Sync
where
    Self: 'static,
{
    fn produces_side_effect(&self) -> bool;
    fn evaluator(&self) -> &NodeEvaluator;
    fn output_promise(&self) -> Arc<OutputPromise>;

    async fn evaluate(self: Arc<Self>);

    async fn ensure_evaluation(self: Arc<Self>) {
        self.evaluator()
            .ensure_evaluation(|| self.clone().evaluate())
            .await;
    }
}
