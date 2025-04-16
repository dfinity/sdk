use std::sync::Arc;

#[async_trait::async_trait]
pub trait Node: Send + Sync
where
    Self: 'static,
{
    async fn evaluate(self: Arc<Self>);
}
