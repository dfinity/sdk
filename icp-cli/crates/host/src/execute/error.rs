use crate::prettify::PrettifyError;
use crate::registry::edge::EdgeType;
use crate::registry::error::NodeConstructorError;
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
