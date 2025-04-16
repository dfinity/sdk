use futures::future::{BoxFuture, Shared};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::OnceCell;

pub struct EvalHandle {
    eval_future: OnceCell<Shared<BoxFuture<'static, ()>>>,
}

impl EvalHandle {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            eval_future: OnceCell::new(),
        })
    }

    pub fn set_eval_future(&self, future: Shared<BoxFuture<'static, ()>>) {
        self.eval_future
            .set(future)
            .expect("eval_future already set");
    }

    pub async fn wait(&self) {
        self.eval_future
            .get()
            .expect("eval_future not set")
            .clone()
            .await;
    }
}

pub struct OutputPromise<T: Clone + Send + 'static + std::fmt::Debug> {
    eval_handle: Arc<EvalHandle>,
    value: OnceCell<T>,
}

impl<T: Clone + Send + 'static + std::fmt::Debug> OutputPromise<T> {
    pub fn new(eval_handle: Arc<EvalHandle>) -> Self {
        Self {
            eval_handle,
            value: OnceCell::new(),
        }
    }

    pub fn set(&self, value: T) {
        self.value.set(value).expect("output already set");
    }

    pub async fn get(&self) -> T {
        // wait for evaluation to complete (if not already)
        self.eval_handle.wait().await;

        // Return the filled value
        self.value
            .get()
            .expect("output should have been set in evaluate()")
            .clone()
    }
}

#[derive(Clone)]
pub enum AnyOutputPromise {
    String(Arc<OutputPromise<String>>),
    //    JsonValue(Arc<OutputPromise<serde_json::Value>>),
    // Add more as needed
}

impl AnyOutputPromise {
    // pub(crate) fn set_owner(&self, p0: Weak<dyn Node>) {
    //     match self {
    //         AnyOutputPromise::String(op) => op.set_owner(p0),
    //     }
    // }
}

#[derive(Debug, Error)]
pub enum PromiseTypeError {
    #[error("Type mismatch: expected String")]
    ExpectedString,
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
