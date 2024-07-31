use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("failed to canonicalize '{path}'")]
pub struct CanonicalizePathError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to copy {from} to {to}")]
pub struct CopyFileError {
    pub from: PathBuf,
    pub to: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to create directory {path} and parents")]
pub struct CreateDirAllError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
pub enum EnsureDirExistsError {
    #[error(transparent)]
    CreateDirAll(#[from] CreateDirAllError),

    #[error("path {0} is not a directory")]
    NotADirectory(PathBuf),
}

#[derive(Error, Debug)]
pub enum EnsureParentDirExistsError {
    #[error(transparent)]
    EnsureDirExists(#[from] EnsureDirExistsError),

    #[error(transparent)]
    NoParentPath(#[from] NoParentPathError),
}

#[derive(Error, Debug)]
#[error("failed to determine parent path for '{0}'")]
pub struct NoParentPathError(pub PathBuf);

#[derive(Error, Debug)]
#[error("failed to read directory {path}")]
pub struct ReadDirError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to read from {path}")]
pub struct ReadFileError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to remove directory {path}")]
pub struct RemoveDirectoryError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to write to {path}")]
pub struct WriteFileError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to read metadata of {path}")]
pub struct ReadMetadataError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to read permissions of {path}")]
pub struct ReadPermissionsError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to read {path} as string")]
pub struct ReadToStringError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
#[error("failed to set permissions of {path}")]
pub struct SetPermissionsError {
    pub path: PathBuf,
    pub source: std::io::Error,
}

#[derive(Error, Debug)]
pub enum SetPermissionsReadWriteError {
    #[error(transparent)]
    ReadPermissions(#[from] ReadPermissionsError),

    #[error(transparent)]
    SetPermissions(#[from] SetPermissionsError),
}

#[derive(Error, Debug)]
pub enum FsErrorKind {
    #[error("Failed to remove directory {0} and its contents")]
    RemoveDirectoryAndContentsFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to remove file {0}")]
    RemoveFileFailed(PathBuf, #[source] std::io::Error),

    #[error("Failed to rename {0} to {1}")]
    RenameFailed(Box<PathBuf>, Box<PathBuf>, #[source] std::io::Error),

    #[error("Failed to unpack archive in {0}")]
    UnpackingArchiveFailed(PathBuf, #[source] std::io::Error),
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
