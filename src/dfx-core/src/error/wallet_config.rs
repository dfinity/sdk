use crate::error::config::ConfigError;
use crate::error::fs::FsError;
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletConfigError {
    #[error("Failed to ensure existence of parent directory for wallet configuration")]
    EnsureWalletConfigDirFailed(#[source] FsError),

    #[error("Failed to get wallet configuration path")]
    GetWalletConfigPathFailed(Box<String>, Box<String>, #[source] ConfigError),

    #[error("Failed to load wallet configuration")]
    LoadWalletConfigFailed(#[source] StructuredFileError),

    #[error("Failed to save wallet configuration")]
    SaveWalletConfigFailed(#[source] StructuredFileError),
}
