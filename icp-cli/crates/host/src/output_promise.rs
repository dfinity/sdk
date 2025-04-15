use crate::node::Node;
use crate::value::OutputValue;
use futures::future::FutureExt;
use futures::future::{BoxFuture, Shared};
use std::sync::Weak;
use tokio::sync::OnceCell;

pub struct OutputPromise {
    future: OnceCell<Shared<BoxFuture<'static, OutputValue>>>,
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

    pub async fn get(&self) -> OutputValue {
        if self.future.get().is_none() {
            let node = self
                .owner
                .get()
                .expect("no owner")
                .upgrade()
                .expect("owner dropped");

            node.ensure_evaluation().await;
        }

        self.future.get().unwrap().clone().await
    }

    pub fn set(&self, value: OutputValue) {
        // Build a ready future wrapping the value, store it as shared future
        let fut = async move { value }.boxed().shared();
        self.future.set(fut).unwrap();
    }
}
