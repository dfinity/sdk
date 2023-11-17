use crate::error::fs::FsError;
use crate::error::get_user_home::GetUserHomeError;
use std::path::PathBuf;
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

#[derive(Error, Debug)]
pub enum GetOutputEnvFileError {
    #[error("failed to canonicalize output_env_file")]
    Canonicalize(#[source] FsError),

    #[error("The output_env_file must be within the project root, but is {}", .0.display())]
    OutputEnvFileMustBeInProjectRoot(PathBuf),

    #[error("The output_env_file must be a relative path, but is {}", .0.display())]
    OutputEnvFileMustBeRelative(PathBuf),

    #[error(transparent)]
    Parent(FsError),
}
