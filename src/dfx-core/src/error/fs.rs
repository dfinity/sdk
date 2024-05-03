use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsErrorKind {
    #[error("Failed to canonicalize {0}")]
    CanonicalizePathFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to copy {0} to {1}")]
    CopyFileFailed(Box<PathBuf>, Box<PathBuf>, #[source] std::io::Error),

    #[error("Failed to create {0}")]
    CreateDirectoryFailed(PathBuf, #[source] std::io::Error),

    #[error("Cannot determine parent folder for {0}")]
    NoParent(PathBuf),

    #[error("Path {0} is not a directory")]
    NotADirectory(PathBuf),

    #[error("Failed to read directory {0}")]
    ReadDirFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to read {0}")]
    ReadFileFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to read metadata of {0}")]
    ReadMetadataFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to read permissions of {0}")]
    ReadPermissionsFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to read {0} as string")]
    ReadToStringFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to remove directory {0}")]
    RemoveDirectoryFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to remove directory {0} and its contents")]
    RemoveDirectoryAndContentsFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to remove file {0}")]
    RemoveFileFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to rename {0} to {1}")]
    RenameFailed(Box<PathBuf>, Box<PathBuf>, #[source] std::io::Error),

    #[error("Failed to unpack archive in {0}")]
    UnpackingArchiveFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to write to {0}")]
    WriteFileFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to set permissions of {0}")]
    WritePermissionsFailed(PathBuf, #[source] std::io::Error),
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct FsError(pub Box<FsErrorKind>);

impl FsError {
    pub fn new(kind: FsErrorKind) -> Self {
        FsError(Box::new(kind))
    }
}

impl<E> From<E> for FsError
where
    FsErrorKind: From<E>,
{
    fn from(err: E) -> Self {
        FsError(Box::new(FsErrorKind::from(err)))
    }
}
