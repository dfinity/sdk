use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoErrorKind {
    #[error("Failed to canonicalize {0}: {1}")]
    CanonicalizePathFailed(PathBuf, std::io::Error),

    #[error("Failed to copy {0} to {1}: {2}")]
    CopyFileFailed(Box<PathBuf>, Box<PathBuf>, std::io::Error),

    #[error("Failed to create {0}: {1}")]
    CreateDirectoryFailed(PathBuf, std::io::Error),

    #[error("Cannot determine parent folder for {0}")]
    NoParent(PathBuf),

    #[error("Path {0} is not a directory")]
    NotADirectory(PathBuf),

    #[error("Failed to read directory {0}: {1}")]
    ReadDirFailed(PathBuf, std::io::Error),

    #[error("Failed to read {0}: {1}")]
    ReadFileFailed(PathBuf, std::io::Error),

    #[error("Failed to read permissions of {0}: {1}")]
    ReadPermissionsFailed(PathBuf, std::io::Error),

    #[error("Failed to read {0} as string: {1}")]
    ReadToStringFailed(PathBuf, std::io::Error),

    #[error("Failed to remove directory {0}: {1}")]
    RemoveDirectoryFailed(PathBuf, std::io::Error),

    #[error("Failed to remove directory {0} and its contents: {1}")]
    RemoveDirectoryAndContentsFailed(PathBuf, std::io::Error),

    #[error("Failed to remove file {0}: {1}")]
    RemoveFileFailed(PathBuf, std::io::Error),

    #[error("Failed to rename {0} to {1}: {2}")]
    RenameFailed(Box<PathBuf>, Box<PathBuf>, std::io::Error),

    #[error("Failed to write to {0}: {1}")]
    WriteFileFailed(PathBuf, std::io::Error),

    #[error("Failed to set permissions of {0}: {1}")]
    WritePermissionsFailed(PathBuf, std::io::Error),
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct IoError(pub Box<IoErrorKind>);

impl IoError {
    pub fn new(kind: IoErrorKind) -> Self {
        IoError(Box::new(kind))
    }
}

impl<E> From<E> for IoError
where
    IoErrorKind: From<E>,
{
    fn from(err: E) -> Self {
        IoError(Box::new(IoErrorKind::from(err)))
    }
}
