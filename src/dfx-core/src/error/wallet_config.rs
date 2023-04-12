use crate::error::config::ConfigError;
use crate::error::fs::FsError;
use crate::error::structured_file::StructuredFileError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletConfigError {
    #[error("Failed to ensure existence of parent directory for wallet configuration: {0}.")]
    EnsureWalletConfigDirFailed(FsError),

    #[error("Failed to get wallet configuration path: {0}")]
    GetWalletConfigPathFailed(Box<String>, Box<String>, ConfigError),

    #[error("Failed to load wallet configuration: {0}.")]
    LoadWalletConfigFailed(StructuredFileError),

    #[error("Failed to save wallet configuration: {0}.")]
    SaveWalletConfigFailed(StructuredFileError),
}
