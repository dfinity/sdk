use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Failed to create {0}: {1}")]
    CreateDirectoryFailed(PathBuf, std::io::Error),

    #[error("Cannot determine parent folder for {0}")]
    NoParent(PathBuf),

    #[error("Failed to read permissions of {0}: {1}")]
    ReadPermissionsFailed(PathBuf, std::io::Error),

    #[error("Failed to rename {0} to {1}: {2}")]
    RenameFailed(PathBuf, PathBuf, std::io::Error),

    #[error("Failed to write to {0}: {1}")]
    WriteFileFailed(PathBuf, std::io::Error),

    #[error("Failed to set permissions of {0}: {1}")]
    WritePermissionsFailed(PathBuf, std::io::Error),
}
