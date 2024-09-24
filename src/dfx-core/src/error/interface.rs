use crate::error::extension::NewExtensionManagerError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewExtensionManagerFromCachePathError {
    #[error("no filename in cache path '{0}'")]
    NoCachePathFilename(PathBuf),

    #[error("filename in cache path '{0}' is not valid UTF-8")]
    CachePathFilenameNotUtf8(PathBuf),

    #[error("cannot parse version from cache path filename")]
    ParseVersion(#[from] semver::Error),

    #[error(transparent)]
    NewExtensionManager(#[from] NewExtensionManagerError),
}
