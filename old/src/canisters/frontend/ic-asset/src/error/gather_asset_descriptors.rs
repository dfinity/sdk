use crate::error::get_asset_config::GetAssetConfigError;
use crate::error::load_config::AssetLoadConfigError;
use dfx_core::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

/// Errors related to building asset list and reading asset configurations.
#[derive(Error, Debug)]
pub enum GatherAssetDescriptorsError {
    /// An asset with a given key exists in more than one source directory.
    #[error("Asset with key '{0}' defined at {1} and {2}")]
    DuplicateAssetKey(String, Box<PathBuf>, Box<PathBuf>),

    /// Failed to get asset configuration.
    #[error("Failed to get asset configuration: {0}")]
    GetAssetConfigFailed(#[from] GetAssetConfigError),

    /// Failed to canonicalize a directory entry.
    #[error("Invalid directory entry: {0}")]
    InvalidDirectoryEntry(FsError),

    /// Failed to canonicalize a source directory.
    #[error("Invalid source directory: {0}")]
    InvalidSourceDirectory(FsError),

    /// Failed to load the asset configuration for a directory.
    #[error("Failed to load asset configuration: {0}")]
    LoadConfigFailed(AssetLoadConfigError),
}
