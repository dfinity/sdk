use crate::error::cache::GetVersionFromCachePathError;
use crate::error::extension::NewExtensionManagerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewExtensionManagerFromCachePathError {
    #[error(transparent)]
    GetVersionFromCachePath(#[from] GetVersionFromCachePathError),

    #[error(transparent)]
    NewExtensionManager(#[from] NewExtensionManagerError),
}
