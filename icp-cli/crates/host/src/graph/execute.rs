use std::sync::Arc;

#[async_trait::async_trait]
pub trait Execute: Send + Sync
where
    Self: 'static,
{
    async fn execute(self: Arc<Self>);
}
