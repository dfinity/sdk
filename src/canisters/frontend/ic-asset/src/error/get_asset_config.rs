use dfx_core::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GetAssetConfigError {
    #[error("No configuration found for asset '{0}'")]
    AssetConfigNotFound(PathBuf),

    #[error("Invalid asset path: {0}")]
    InvalidPath(FsError),
}
