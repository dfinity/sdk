use crate::error::archive::GetArchivePathError;
use crate::error::fs::{
    CreateDirAllError, ReadDirError, ReadFileError, ReadPermissionsError,
    RemoveDirectoryAndContentsError, SetPermissionsError, UnpackingArchiveError, WriteFileError,
};
use crate::error::get_current_exe::GetCurrentExeError;
use crate::error::get_user_home::GetUserHomeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetBinaryCommandPathError {
    #[error(transparent)]
    Install(#[from] InstallCacheError),

    #[error(transparent)]
    GetBinaryPathFromVersion(#[from] GetBinaryPathFromVersionError),
}

#[derive(Error, Debug)]
pub enum GetBinaryPathFromVersionError {
    #[error(transparent)]
    GetBinCache(CacheError),
}

#[derive(Error, Debug)]
pub enum ListCacheVersionsError {
    #[error(transparent)]
    ReadDir(#[from] ReadDirError),

    #[error(transparent)]
    GetBinCacheRoot(CacheError),

    #[error("failed to parse '{0}' as Semantic Version")]
    MalformedSemverString(String, #[source] semver::Error),

    #[error("failed to read entry in cache directory")]
    ReadCacheEntryFailed(#[source] std::io::Error),
}

#[derive(Error, Debug)]
pub enum InstallCacheError {
    #[error(transparent)]
    CreateDirAll(#[from] CreateDirAllError),

    #[error(transparent)]
    GetArchivePath(#[from] GetArchivePathError),

    #[error(transparent)]
    GetBinCache(CacheError),

    #[error(transparent)]
    GetCurrentExeError(#[from] GetCurrentExeError),

    #[error("invalid cache for version '{0}'.")]
    InvalidCacheForDfxVersion(String),

    #[error("failed to parse '{0}' as Semantic Version")]
    MalformedSemverString(String, #[source] semver::Error),

    #[error("failed to iterate through binary cache")]
    ReadBinaryCacheEntriesFailed(#[source] std::io::Error),

    #[error("failed to read binary cache entry")]
    ReadBinaryCacheEntryFailed(#[source] std::io::Error),

    #[error("failed to read binary cache")]
    ReadBinaryCacheStoreFailed(#[source] std::io::Error),

    #[error(transparent)]
    ReadFile(#[from] ReadFileError),

    #[error(transparent)]
    ReadPermissions(#[from] ReadPermissionsError),

    #[error(transparent)]
    RemoveDirectoryAndContents(#[from] RemoveDirectoryAndContentsError),

    #[error(transparent)]
    SetPermissions(#[from] SetPermissionsError),

    #[error(transparent)]
    UnpackingArchive(#[from] UnpackingArchiveError),

    #[error(transparent)]
    WriteFile(#[from] WriteFileError),
}

#[derive(Error, Debug)]
pub enum CacheError {
    #[error(transparent)]
    GetUserHomeError(#[from] GetUserHomeError),

    #[error(transparent)]
    RemoveDirectoryAndContents(#[from] RemoveDirectoryAndContentsError),

    #[error("failed to create cache directory")]
    CreateCacheDirectoryFailed(#[source] CreateDirAllError),

    #[error("Cannot find cache directory at '{0}'.")]
    FindCacheDirectoryFailed(std::path::PathBuf),
}
