use crate::host::prettify::PrettifyError;
use crate::workflow::registry::edge::EdgeType;
use crate::workflow::registry::error::NodeConstructorError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphExecutionError {
    #[error(transparent)]
    PrettifyError(#[from] PrettifyError),
}

#[derive(thiserror::Error, Debug)]
pub enum StringPromiseError {
    #[error("Expected a String promise, but got {got:?}")]
    TypeMismatch { got: EdgeType },
}

#[derive(thiserror::Error, Debug)]
pub enum WasmPromiseError {
    #[error("Expected a Wasm promise, but got {got:?}")]
    TypeMismatch { got: EdgeType },
}

#[derive(Error, Debug)]
pub enum ExecutionGraphFromPlanError {
    #[error(transparent)]
    NodeConstructorError(#[from] NodeConstructorError),
}
