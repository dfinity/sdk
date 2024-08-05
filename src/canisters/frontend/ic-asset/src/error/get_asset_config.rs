use dfx_core::error::fs::NoParentPathError;
use std::path::PathBuf;
use thiserror::Error;

/// Errors related to getting asset configuration.
#[derive(Error, Debug)]
pub enum GetAssetConfigError {
    /// An asset exists, but it does not have a configuration.
    #[error("No configuration found for asset '{0}'")]
    AssetConfigNotFound(PathBuf),

    /// The path to an asset was invalid.
    #[error("invalid asset path")]
    InvalidPath(#[from] NoParentPathError),
}
