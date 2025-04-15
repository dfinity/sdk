use futures_util::future::FutureExt;
use futures_util::future::{BoxFuture, Shared};
use tokio::sync::OnceCell;

pub struct NodeEvaluator {
    eval_future: OnceCell<Shared<BoxFuture<'static, ()>>>,
}

impl NodeEvaluator {
    pub fn new() -> Self {
        Self {
            eval_future: OnceCell::new(),
        }
    }

    pub async fn ensure_evaluation<Fut>(&self, eval_fn: impl FnOnce() -> Fut)
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let eval_future = self
            .eval_future
            .get_or_init(|| async move { eval_fn().boxed().shared() })
            .await;
        eval_future.clone().await;
    }
}
