use crate::node::Node;
use crate::value::Value;
use futures::future::FutureExt;
use futures::future::{BoxFuture, Shared};
use std::sync::Weak;
use tokio::sync::OnceCell;

pub struct OutputPromise {
    future: OnceCell<Shared<BoxFuture<'static, Value>>>,
    owner: OnceCell<Weak<dyn Node>>,
}

impl OutputPromise {
    pub fn new() -> Self {
        Self {
            future: OnceCell::new(),
            owner: OnceCell::new(),
        }
    }

    pub fn set_owner(&self, owner: Weak<dyn Node>) {
        self.owner.set(owner).expect("Owner already set");
    }

    pub async fn get(&self) -> Value {
        if self.future.get().is_none() {
            if let Some(node) = self.owner.get().expect("no owner").upgrade() {
                node.ensure_evaluation().await;
            } else {
                panic!("Owning node dropped");
            }
        }

        self.future.get().unwrap().clone().await
    }

    pub fn set(&self, value: Value) {
        // Build a ready future wrapping the value, store it as shared future
        let fut = async move { value }.boxed().shared();
        self.future.set(fut).unwrap();
    }
}
