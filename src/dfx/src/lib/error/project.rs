use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectError {
    // #[error(transparent)]
    // FoundationError(#[from] dfx_core::error::foundation::FoundationError),

    // #[error(transparent)]
    // IoError(#[from] dfx_core::error::io::IoError),

    // #[error(transparent)]
    // ProcessError(#[from] dfx_core::error::process::ProcessError),

    // #[error("Cannot create cache directory: {0}")]
    // CreateCacheDirectoryFailed(dfx_core::error::io::IoError),

    // #[error("Cannot find cache directory at '{0}'.")]
    // FindCacheDirectoryFailed(std::path::PathBuf),

    // #[error("Invalid cache for version '{0}'.")]
    // InvalidCacheForDfxVersion(String),

    // #[error("Unable to parse '{0}' as Semantic Version: {1}")]
    // MalformedSemverString(String, semver::Error),

    // #[error("Failed to read binary cache: {0}")]
    // ReadBinaryCacheStoreFailed(std::io::Error),

    // #[error("Failed to iterate through binary cache: {0}")]
    // ReadBinaryCacheEntriesFailed(std::io::Error),

    // #[error("Failed to read binary cache entry: {0}")]
    // ReadBinaryCacheEntryFailed(std::io::Error),

    // #[error("Failed to read entry in cache directory: {0}")]
    // ReadCacheEntryFailed(std::io::Error),
    #[error("dummy sucker")]
    Dummy,
}
