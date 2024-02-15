use crate::error::extension::{GetExtensionByNameError, ProcessCanisterDeclarationError};
use crate::error::fs::FsError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

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
    #[error(transparent)]
    GetExtensionByName(#[from] GetExtensionByNameError),

    #[error(transparent)]
    ProcessCanisterDeclarationError(#[from] ProcessCanisterDeclarationError),
}
