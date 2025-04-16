use crate::prettify::PrettifyError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphExecutionError {
    #[error(transparent)]
    PrettifyError(#[from] PrettifyError),
    // You can add more as needed, like:
    // #[error(transparent)]
    // SomeOtherError(#[from] SomeOtherType),
}
