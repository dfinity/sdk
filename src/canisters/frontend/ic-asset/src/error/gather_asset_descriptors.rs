use crate::error::get_asset_config::GetAssetConfigError;
use crate::error::load_config::AssetLoadConfigError;

use dfx_core::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GatherAssetDescriptorsError {
    #[error("Asset with key '{0}' defined at {1} and {2}")]
    DuplicateAssetKey(String, Box<PathBuf>, Box<PathBuf>),

    #[error("Failed to get asset configuration: {0}")]
    GetAssetConfigFailed(#[from] GetAssetConfigError),

    #[error("Invalid directory entry: {0}")]
    InvalidDirectoryEntry(FsError),

    #[error("Invalid source directory: {0}")]
    InvalidSourceDirectory(FsError),

    #[error("Failed to load asset configuration: {0}")]
    LoadConfigFailed(AssetLoadConfigError),
}
