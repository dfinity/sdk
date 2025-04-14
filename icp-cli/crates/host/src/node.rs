use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

#[async_trait::async_trait]
pub trait Node: Send + Sync {
    fn produces_side_effect(&self) -> bool;
    async fn evaluate(&self);
}
