use crate::error::archive::GetArchivePathError;
use crate::error::fs::{
    CreateDirAllError, ReadDirError, ReadFileError, ReadPermissionsError,
    RemoveDirectoryAndContentsError, SetPermissionsError, UnpackingArchiveError, WriteFileError,
};
use crate::error::get_current_exe::GetCurrentExeError;
use crate::error::get_user_home::GetUserHomeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error(transparent)]
    Archive(#[from] GetArchivePathError),

    #[error(transparent)]
    CreateDirAll(#[from] CreateDirAllError),

    #[error(transparent)]
    GetCurrentExeError(#[from] GetCurrentExeError),

    #[error(transparent)]
    GetUserHomeError(#[from] GetUserHomeError),

    #[error(transparent)]
    UnpackingArchive(#[from] UnpackingArchiveError),

    #[error(transparent)]
    ReadFile(#[from] ReadFileError),

    #[error(transparent)]
    ReadPermissions(#[from] ReadPermissionsError),

    #[error(transparent)]
    RemoveDirectoryAndContents(#[from] RemoveDirectoryAndContentsError),

    #[error(transparent)]
    SetPermissions(#[from] SetPermissionsError),

    #[error(transparent)]
    WriteFile(#[from] WriteFileError),

    #[error(transparent)]
    ProcessError(#[from] crate::error::process::ProcessError),

    #[error("failed to create cache directory")]
    CreateCacheDirectoryFailed(#[source] CreateDirAllError),

    #[error("Cannot find cache directory at '{0}'.")]
    FindCacheDirectoryFailed(std::path::PathBuf),

    #[error("Invalid cache for version '{0}'.")]
    InvalidCacheForDfxVersion(String),

    #[error("Unable to parse '{0}' as Semantic Version")]
    MalformedSemverString(String, #[source] semver::Error),

    #[error("Failed to read binary cache")]
    ReadBinaryCacheStoreFailed(#[source] std::io::Error),

    #[error("Failed to iterate through binary cache")]
    ReadBinaryCacheEntriesFailed(#[source] std::io::Error),

    #[error("Failed to read binary cache entry")]
    ReadBinaryCacheEntryFailed(#[source] std::io::Error),

    #[error("Failed to read entry in cache directory")]
    ReadCacheEntryFailed(#[source] std::io::Error),

    #[error(transparent)]
    ReadDir(#[from] ReadDirError),
}
