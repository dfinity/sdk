use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cannot create cache directory at '{0}'.")]
    CreateCacheDirectoryFailed(dfx_core::error::io::IoError),

    #[error("Cannot find cache directory at '{0}'.")]
    FindCacheDirectoryFailed(std::path::PathBuf),

    #[error(transparent)]
    FoundationError(#[from] dfx_core::error::foundation::FoundationError),

    #[error("Unknown version '{0}'.")]
    UnknownVersion(String),

    #[error("Failed to parse version from '{0}'.")]
    MalformedSemverVersion(semver::Error),

    #[error("Failed to read binary cache: '{0}'.")]
    ReadBinaryCacheFailed(std::io::Error),

    #[error("Failed to iterate through binary cache: {0}")]
    ReadBinaryCacheEntriesFailed(std::io::Error),

    #[error("Failed to read binary cache entry: '{0}'.")]
    ReadBinaryCacheEntryFailed(std::io::Error),

    #[error("Failed to read entry in cache directory: '{0}'.")]
    ReadCacheEntryFailed(std::io::Error),

    #[error(transparent)]
    IoError(#[from] dfx_core::error::io::IoError),

    #[error(transparent)]
    ProcessError(#[from] dfx_core::error::process::ProcessError),
}
