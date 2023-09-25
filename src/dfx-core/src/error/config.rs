use crate::error::fs::FsError;
use crate::error::get_user_home::GetUserHomeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to ensure config directory exists: {0}")]
    EnsureConfigDirectoryExistsFailed(FsError),

    #[error("Failed to determine config directory path: {0}")]
    DetermineConfigDirectoryFailed(GetUserHomeError),

    #[error("Failed to determine shared network data directory: {0}")]
    DetermineSharedNetworkDirectoryFailed(GetUserHomeError),
}
