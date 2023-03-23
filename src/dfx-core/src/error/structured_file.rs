use crate::error::io::IoError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructuredFileError {
    #[error("Failed to parse contents of {0} as json: {1}")]
    DeserializeJsonFileFailed(Box<PathBuf>, serde_json::Error),

    #[error("Failed to parse contents of json: {0}")]
    DeserializeJsonContentFailed(serde_json::Error),

    #[error("Failed to read JSON file: {0}")]
    ReadJsonFileFailed(IoError),

    #[error("Failed to serialize JSON to {0}: {1}")]
    SerializeJsonFileFailed(Box<PathBuf>, serde_json::Error),

    #[error("Failed to write JSON file: {0}")]
    WriteJsonFileFailed(IoError),

    #[error("Failed to pretty-print '{0}' as JSON: {1}")]
    PrettyPrintAsJsonFailed(String, serde_json::Error),
}
