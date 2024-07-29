use crate::error::config::ConfigError;
use crate::error::fs::{CreateDirAllError, EnsureParentDirExistsError};
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletConfigError {
    #[error("failed to ensure existence of parent directory for wallet configuration")]
    EnsureWalletConfigDirFailed(#[source] CreateDirAllError),

    #[error("Failed to get wallet configuration path")]
    GetWalletConfigPathFailed(Box<String>, Box<String>, #[source] ConfigError),

    #[error("Failed to load wallet configuration")]
    LoadWalletConfigFailed(#[source] StructuredFileError),

    #[error("Failed to save wallet configuration")]
    SaveWalletConfig(#[source] SaveWalletConfigError),
}

#[derive(Error, Debug)]
pub enum SaveWalletConfigError {
    #[error(transparent)]
    EnsureParentDirExists(#[from] EnsureParentDirExistsError),

    #[error(transparent)]
    SaveJsonFile(#[from] StructuredFileError),
}
