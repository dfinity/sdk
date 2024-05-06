use crate::error::load_rule::LoadRuleError;
use dfx_core::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

/// Errors related to loading asset configuration.
#[derive(Error, Debug)]
pub enum AssetLoadConfigError {
    /// Failed a filesystem operation; the inner error contains the details.
    #[error(transparent)]
    FsError(#[from] FsError),

    /// Failed to canonicalize the root directory.
    #[error("root_dir '{0}' is expected to be a canonical path")]
    InvalidRootDir(PathBuf),

    /// Failed to load a rule from the asset configuration file.
    #[error("Failed to load rule in {0}: {1}")]
    LoadRuleFailed(PathBuf, LoadRuleError),

    /// An asset configuration file was not valid JSON5.
    #[error("Malformed JSON asset config file '{0}': {1}")]
    MalformedAssetConfigFile(PathBuf, json5::Error),

    /// both `assets.json` and `assets.json5` files exist in the same directory.
    #[error("both {} and {} files exist in the same directory (dir = {:?})",
    crate::asset::config::ASSETS_CONFIG_FILENAME_JSON,
    crate::asset::config::ASSETS_CONFIG_FILENAME_JSON5,
    .0.display())]
    MultipleConfigurationFiles(PathBuf),
}
