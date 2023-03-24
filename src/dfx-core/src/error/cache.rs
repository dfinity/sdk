use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error(transparent)]
    FoundationError(#[from] crate::error::foundation::FoundationError),

    #[error(transparent)]
    IoError(#[from] crate::error::fs::FsError),

    #[error(transparent)]
    ProcessError(#[from] crate::error::process::ProcessError),

    #[error("Cannot create cache directory: {0}")]
    CreateCacheDirectoryFailed(crate::error::fs::FsError),

    #[error("Cannot find cache directory at '{0}'.")]
    FindCacheDirectoryFailed(std::path::PathBuf),

    #[error("Invalid cache for version '{0}'.")]
    InvalidCacheForDfxVersion(String),

    #[error("Unable to parse '{0}' as Semantic Version: {1}")]
    MalformedSemverString(String, semver::Error),

    #[error("Failed to read binary cache: {0}")]
    ReadBinaryCacheStoreFailed(std::io::Error),

    #[error("Failed to iterate through binary cache: {0}")]
    ReadBinaryCacheEntriesFailed(std::io::Error),

    #[error("Failed to read binary cache entry: {0}")]
    ReadBinaryCacheEntryFailed(std::io::Error),

    #[error("Failed to read entry in cache directory: {0}")]
    ReadCacheEntryFailed(std::io::Error),
}
