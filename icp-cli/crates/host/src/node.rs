use crate::node_state::NodeEvaluator;
use crate::output_promise::OutputPromise;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait Node: Send + Sync
where
    Self: 'static,
{
    fn evaluator(&self) -> &NodeEvaluator;
    // fn output_promise(&self) -> Arc<OutputPromise<T>>;

    async fn evaluate(self: Arc<Self>);

    async fn ensure_evaluation(self: Arc<Self>) {
        self.evaluator()
            .ensure_evaluation(|| self.clone().evaluate())
            .await;
    }
}
