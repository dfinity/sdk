use crate::error::structured_file::StructuredFileError;

use crate::error::io::IoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletConfigError {
    #[error("Failed to ensure existence of parent directory for wallet configuration: {0}.")]
    EnsureWalletConfigDirFailed(IoError),

    #[error("Failed to load wallet configuration: {0}.")]
    LoadWalletConfigFailed(StructuredFileError),

    #[error("Failed to save wallet configuration: {0}.")]
    SaveWalletConfigFailed(StructuredFileError),
}
