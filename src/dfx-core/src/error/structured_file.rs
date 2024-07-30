use crate::error::fs::{FsError, ReadFileError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructuredFileError {
    #[error("Failed to parse contents of {0} as json")]
    DeserializeJsonFileFailed(Box<PathBuf>, #[source] serde_json::Error),

    #[error("Failed to read JSON file")]
    ReadJsonFileFailed(#[from] ReadFileError),

    #[error("Failed to serialize JSON to {0}")]
    SerializeJsonFileFailed(Box<PathBuf>, #[source] serde_json::Error),

    #[error("Failed to write JSON file")]
    WriteJsonFileFailed(#[source] FsError),
}
