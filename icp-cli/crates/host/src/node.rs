use std::sync::Arc;

#[async_trait::async_trait]
pub trait Node: Send + Sync {
    fn produces_side_effect(&self) -> bool;

    async fn evaluate(self: Arc<Self>);

    // fn evaluation_cell(&self) -> &OnceCell<Shared<BoxFuture<'static, ()>>>;

    async fn ensure_evaluation(self: Arc<Self>);
}
