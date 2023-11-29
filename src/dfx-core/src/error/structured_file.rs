use crate::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

use super::extension::ExtensionError;

#[derive(Error, Debug)]
pub enum StructuredFileError {
    #[error("Failed to parse contents of {0} as json: {1}")]
    DeserializeJsonFileFailed(Box<PathBuf>, serde_json::Error),

    #[error("Failed to read JSON file: {0}")]
    ReadJsonFileFailed(FsError),

    #[error("Failed to serialize JSON to {0}: {1}")]
    SerializeJsonFileFailed(Box<PathBuf>, serde_json::Error),

    #[error("Failed to write JSON file: {0}")]
    WriteJsonFileFailed(FsError),
}

#[derive(Error, Debug)]
pub enum ReadConfigurationError {
    #[error(transparent)]
    StructuredFile(#[from] StructuredFileError),
    #[error(transparent)]
    TransformConfiguration(#[from] TransformConfigurationError),
}

#[derive(Error, Debug)]
pub enum TransformConfigurationError {
    #[error("Configuration transformation failed: {0}")]
    ConfigurationTransformationFailed(String), // Or another error type if necessary
    #[error("Extension error: {0}")]
    ExtensionError(#[from] ExtensionError), // Note that `from` here allows automatic conversion
}
