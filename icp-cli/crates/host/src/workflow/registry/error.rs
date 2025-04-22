use crate::workflow::execute::error::StringPromiseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StringSourceError {
    #[error("Missing input: {0}")]
    MissingInput(String),

    #[error(transparent)]
    StringPromiseError(#[from] StringPromiseError),
}

#[derive(Error, Debug)]
pub enum NodeConstructorError {
    #[error(transparent)]
    StringSource(#[from] StringSourceError),
}
