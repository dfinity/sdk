use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cannot create cache directory at '{0}'.")]
    CreateCacheDirectoryFailed(PathBuf),

    #[error("Cannot find cache directory at '{0}'.")]
    FindCacheDirectoryFailed(PathBuf),

    #[error(transparent)]
    FoundationError(#[from] dfx_core::error::foundation::FoundationError),

    #[error("Unknown version '{0}'.")]
    UnknownVersion(String),
}
