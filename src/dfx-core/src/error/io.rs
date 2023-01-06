use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Failed to create {0}: {1}")]
    CreateDirectoryFailed(PathBuf, std::io::Error),

    #[error("Failed to rename {0} to {1}: {2}")]
    RenameFailed(PathBuf, PathBuf, std::io::Error),
}
