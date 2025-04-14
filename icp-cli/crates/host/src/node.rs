use futures_util::future::{BoxFuture, Shared};
use std::cell::OnceCell;
use std::future::Future;
use std::sync::Arc;
//use futures_util::future::{BoxFuture, Shared};
use futures_util::future::FutureExt;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

#[async_trait::async_trait]
pub trait Node: Send + Sync {
    fn produces_side_effect(&self) -> bool;

    async fn evaluate(self: Arc<Self>);

    // fn evaluation_cell(&self) -> &OnceCell<Shared<BoxFuture<'static, ()>>>;

    async fn ensure_evaluation(self: Arc<Self>);
}
