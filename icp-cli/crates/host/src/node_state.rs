use crate::node::Node;
use futures_util::future::FutureExt;
use futures_util::future::{BoxFuture, Shared};
use std::sync::Arc;
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

    pub async fn ensure_evaluation(&self, node: Arc<dyn Node>) {
        let eval_future = self
            .eval_future
            .get_or_init(|| async move { node.evaluate().boxed().shared() })
            .await
            .clone();
        eval_future.await;
    }
}
