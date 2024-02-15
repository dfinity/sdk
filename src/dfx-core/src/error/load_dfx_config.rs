use crate::error::fs::FsError;
use crate::error::structured_file::{StructuredFileError};
use thiserror::Error;
use crate::error::extension::ExtensionError;

#[derive(Error, Debug)]
pub enum LoadDfxConfigError {
    #[error("Failed to resolve config path: {0}")]
    ResolveConfigPathFailed(FsError),

    #[error("Failed to determine current working dir: {0}")]
    DetermineCurrentWorkingDirFailed(std::io::Error),

    #[error("Failed to load dfx configuration: {0}")]
    LoadFromFileFailed(ReadConfigurationError),
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
