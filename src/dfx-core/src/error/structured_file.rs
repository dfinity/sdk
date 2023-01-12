use crate::error::io::IoError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructuredFileError {
    #[error("Failed to parse contents of {0} as json: {1}")]
    DeserializeJsonFileFailed(PathBuf, serde_json::Error),

    #[error("Failed to read JSON file: {0}")]
    ReadJsonFileFailed(IoError),

    #[error("Failed to serialize JSON to {0}: {1}")]
    SerializeJsonFileFailed(PathBuf, serde_json::Error),

    #[error("Failed to write JSON file: {0}")]
    WriteJsonFileFailed(IoError),
}
