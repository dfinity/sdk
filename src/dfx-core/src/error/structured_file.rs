use crate::error::io::IoError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructuredFileError {
    #[error("Failed to read JSON file: {0}")]
    ReadJsonFileFailed(IoError),

    #[error("Failed to deserialize JSON from {0}: {1}")]
    DeserializeJsonFileFailed(PathBuf, serde_json::Error),
}
