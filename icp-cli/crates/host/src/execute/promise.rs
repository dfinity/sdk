use crate::execute::execute::SharedExecuteResult;
use crate::execute::GraphExecutionError;
use async_trait::async_trait;
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

#[async_trait]
pub trait Input<T: Clone + Send + Sync + 'static + std::fmt::Debug> {
    async fn get(&self) -> Result<T, Arc<GraphExecutionError>>;
}

pub trait Output<T: Clone + Send + Sync + 'static + std::fmt::Debug>:
    Send + Sync + 'static
{
    fn set(&self, value: T);
}

pub type InputRef<T> = Arc<dyn Input<T> + Send + Sync>;
pub type OutputRef<T> = Arc<dyn Output<T> + Send + Sync>;

pub struct Promise<T: Clone + Send + 'static + std::fmt::Debug> {
    execute_handle: Arc<ExecuteHandle>,
    value: OnceCell<T>,
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static + std::fmt::Debug> Input<T> for Promise<T> {
    async fn get(&self) -> Result<T, Arc<GraphExecutionError>> {
        self.execute_handle.wait().await?;
        Ok(self
            .value
            .get()
            .expect("output should have been set in execute()")
            .clone())
    }
}

impl<T: Clone + Send + Sync + 'static + std::fmt::Debug> Output<T> for Promise<T> {
    fn set(&self, value: T) {
        self.value.set(value).expect("output already set");
    }
}

impl<T: Clone + Send + 'static + std::fmt::Debug> Promise<T> {
    pub fn new(execute_handle: Arc<ExecuteHandle>) -> Self {
        Self {
            execute_handle,
            value: OnceCell::new(),
        }
    }
}

#[derive(Clone)]
pub enum AnyPromise {
    String(Arc<Promise<String>>),
    //    JsonValue(Arc<OutputPromise<serde_json::Value>>),
    // Add more as needed
}

#[derive(Debug, Error)]
pub enum PromiseTypeError {
    #[error("Type mismatch: expected String")]
    ExpectedString,
}
impl AnyPromise {
    pub fn string(&self) -> Result<Arc<Promise<String>>, PromiseTypeError> {
        match self {
            AnyPromise::String(p) => Ok(p.clone()),
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
