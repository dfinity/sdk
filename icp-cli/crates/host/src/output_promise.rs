use crate::graph::execute::SharedExecuteResult;
use crate::graph::GraphExecutionError;
use futures::future::{BoxFuture, Shared};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::OnceCell;

pub struct ExecuteHandle {
    execute_future: OnceCell<Shared<BoxFuture<'static, SharedExecuteResult>>>,
}

impl ExecuteHandle {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            execute_future: OnceCell::new(),
        })
    }

    pub fn set_execute_future(&self, future: Shared<BoxFuture<'static, SharedExecuteResult>>) {
        self.execute_future
            .set(future)
            .expect("eval_future already set");
    }

    pub async fn wait(&self) -> SharedExecuteResult {
        self.execute_future
            .get()
            .expect("eval_future not set")
            .clone()
            .await
    }
}

pub struct OutputPromise<T: Clone + Send + 'static + std::fmt::Debug> {
    execute_handle: Arc<ExecuteHandle>,
    value: OnceCell<T>,
}

impl<T: Clone + Send + 'static + std::fmt::Debug> OutputPromise<T> {
    pub fn new(execute_handle: Arc<ExecuteHandle>) -> Self {
        Self {
            execute_handle,
            value: OnceCell::new(),
        }
    }

    pub fn set(&self, value: T) {
        self.value.set(value).expect("output already set");
    }

    pub async fn get(&self) -> Result<T, Arc<GraphExecutionError>> {
        // wait for execution to complete (if not already)
        self.execute_handle.wait().await?;

        // Return the filled value
        Ok(self
            .value
            .get()
            .expect("output should have been set in execute()")
            .clone())
    }
}

#[derive(Clone)]
pub enum AnyOutputPromise {
    String(Arc<OutputPromise<String>>),
    //    JsonValue(Arc<OutputPromise<serde_json::Value>>),
    // Add more as needed
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
            // _ => Err(PromiseTypeError::ExpectedString),
        }
    }

    // pub fn json_value(&self) -> Result<Arc<OutputPromise<serde_json::Value>>, PromiseTypeError> {
    //     match self {
    //         AnyOutputPromise::JsonValue(p) => Ok(p.clone()),
    //         _ => Err(PromiseTypeError::ExpectedJsonValue),
    //     }
    // }
}
