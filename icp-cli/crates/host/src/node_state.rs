use crate::node::Node;
use futures_util::future::FutureExt;
use futures_util::future::{BoxFuture, Shared};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct NodeState {
    eval_future: OnceCell<Shared<BoxFuture<'static, ()>>>,
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            eval_future: OnceCell::new(),
        }
    }

    pub async fn ensure_evaluation(self: Arc<Self>, node: Arc<dyn Node>) {
        self.eval_future
            .get_or_init(|| async move { node.evaluate().boxed().shared() })
            .await;
    }
}
