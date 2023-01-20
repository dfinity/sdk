use crate::error::foundation::FoundationError;
use crate::error::io::IoError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to ensure config directory exists: {0}")]
    EnsureConfigDirectoryExistsFailed(IoError),

    #[error("Failed to determine config directory path: {0}")]
    DetermineConfigDirectoryFailed(FoundationError),

    #[error("Failed to determine shared network data directory: {0}")]
    DetermineSharedNetworkDirectoryFailed(FoundationError),
}
