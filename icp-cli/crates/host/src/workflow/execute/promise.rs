use crate::workflow::execute::error::{StringPromiseError, WasmPromiseError};
use crate::workflow::execute::execute::SharedExecuteResult;
use crate::workflow::execute::GraphExecutionError;
use crate::workflow::payload::wasm::Wasm;
use crate::workflow::registry::edge::EdgeType;
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
pub trait Input<T: Clone + Send + Sync + 'static> {
    async fn get(&self) -> Result<T, Arc<GraphExecutionError>>;
}

pub trait Output<T: Clone + Send + Sync + 'static>: Send + Sync + 'static {
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
    String(InputRef<String>, Option<OutputRef<String>>),
    Wasm(InputRef<Wasm>, Option<OutputRef<Wasm>>),
    //    JsonValue(Arc<OutputPromise<serde_json::Value>>),
    // Add more as needed
}

impl AnyPromise {
    pub fn string_input(&self) -> Result<InputRef<String>, StringPromiseError> {
        match self {
            AnyPromise::String(expected, _) => Ok(expected.clone()),
            actual => Err(StringPromiseError::TypeMismatch {
                got: actual.edge_type(),
            }),
        }
    }
    pub fn string_output(&self) -> Result<OutputRef<String>, StringPromiseError> {
        match self {
            AnyPromise::String(_, Some(expected)) => Ok(expected.clone()),
            actual => Err(StringPromiseError::TypeMismatch {
                got: actual.edge_type(),
            }),
        }
    }

    pub fn wasm_input(&self) -> Result<InputRef<Wasm>, WasmPromiseError> {
        match self {
            AnyPromise::Wasm(expected, _) => Ok(expected.clone()),
            actual => Err(WasmPromiseError::TypeMismatch {
                got: actual.edge_type(),
            }),
        }
    }

    pub fn wasm_output(&self) -> Result<OutputRef<Wasm>, WasmPromiseError> {
        match self {
            AnyPromise::Wasm(_, Some(expected)) => Ok(expected.clone()),
            actual => Err(WasmPromiseError::TypeMismatch {
                got: actual.edge_type(),
            }),
        }
    }

    fn edge_type(&self) -> EdgeType {
        match self {
            AnyPromise::String(_, _) => EdgeType::String,
            AnyPromise::Wasm(_, _) => EdgeType::Wasm,
        }
    }
}
