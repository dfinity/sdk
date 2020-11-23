use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cannot create cache directory at '{0}'.")]
    CannotCreateCacheDirectory(PathBuf),

    #[error("Cannot find cache directory at '{0}'.")]
    CannotFindCacheDirectory(PathBuf),

    #[error("Cannot find home directory.")]
    CannotFindHomeDirectory(),

    #[error("Unknown version '{0}'.")]
    UnknownVersion(String),
}
