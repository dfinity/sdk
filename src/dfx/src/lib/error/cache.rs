use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cannot create cache directory at '{0}'.")]
    CannotCreateCacheDirectory(PathBuf),

    #[error("Cannot find cache directory at '{0}'.")]
    CannotFindCacheDirectory(PathBuf),

    // Windows paths do not require environment variables (and are found by dirs-next, which has its own errors)
    #[cfg(not(windows))]
    #[error("Cannot find home directory.")]
    CannotFindHomeDirectory(),

    #[error("Unknown version '{0}'.")]
    UnknownVersion(String),
}
