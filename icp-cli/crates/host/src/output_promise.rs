use crate::node::Node;
use crate::value::OutputValue;
use futures::future::FutureExt;
use futures::future::{BoxFuture, Shared};
use std::sync::{Arc, Weak};
use thiserror::Error;
use tokio::sync::OnceCell;

pub struct OutputPromise<T: Clone + Send + 'static> {
    future: OnceCell<Shared<BoxFuture<'static, T>>>,
    owner: OnceCell<Weak<dyn Node>>,
}

impl<T: Clone + Send + 'static> OutputPromise<T> {
    pub fn new() -> Self {
        Self {
            future: OnceCell::new(),
            owner: OnceCell::new(),
        }
    }

    pub fn set_owner(&self, owner: Weak<dyn Node>) {
        self.owner.set(owner).expect("Owner already set");
    }

    pub async fn get(&self) -> T {
        if self.future.get().is_none() {
            let node = self
                .owner
                .get()
                .expect("no owner")
                .upgrade()
                .expect("owner dropped");

            node.ensure_evaluation().await;
        }

        self.future.get().expect("not fulfilled").clone().await
    }

    pub fn set(&self, value: T) {
        // Build a ready future wrapping the value, store it as shared future
        let fut = async move { value }.boxed().shared();
        self.future.set(fut).expect("value already set");
    }
}

#[derive(Clone)]
pub enum AnyOutputPromise {
    String(Arc<OutputPromise<String>>),
    //    JsonValue(Arc<OutputPromise<serde_json::Value>>),
    // Add more as needed
}

impl AnyOutputPromise {
    pub(crate) fn set_owner(&self, p0: Weak<dyn Node>) {
        match self {
            AnyOutputPromise::String(op) => op.set_owner(p0),
        }
    }
}

#[derive(Debug, Error)]
pub enum PromiseTypeError {
    #[error("Type mismatch: expected String")]
    ExpectedString,
    #[error("Type mismatch: expected JsonValue")]
    ExpectedJsonValue,
}
impl AnyOutputPromise {
    pub fn string(&self) -> Result<Arc<OutputPromise<String>>, PromiseTypeError> {
        match self {
            AnyOutputPromise::String(p) => Ok(p.clone()),
            _ => Err(PromiseTypeError::ExpectedString),
        }
    }

    // pub fn json_value(&self) -> Result<Arc<OutputPromise<serde_json::Value>>, PromiseTypeError> {
    //     match self {
    //         AnyOutputPromise::JsonValue(p) => Ok(p.clone()),
    //         _ => Err(PromiseTypeError::ExpectedJsonValue),
    //     }
    // }
}
