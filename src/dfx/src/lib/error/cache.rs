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

    #[error(transparent)]
    IoError(#[from] dfx_core::error::io::IoError),

    #[error("Failed to parse version from '{0}'.")]
    MalformedSemverVersion(semver::Error),

    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
}
