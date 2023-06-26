use crate::error::load_rule::LoadRuleError;

use dfx_core::error::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetLoadConfigError {
    #[error(transparent)]
    FsError(#[from] FsError),

    #[error("root_dir '{0}' is expected to be a canonical path")]
    InvalidRootDir(PathBuf),

    #[error("Failed to load rule in {0}: {1}")]
    LoadRuleFailed(PathBuf, LoadRuleError),

    #[error("Malformed JSON asset config file '{0}': {1}")]
    MalformedAssetConfigFile(PathBuf, json5::Error),

    #[error("both {} and {} files exist in the same directory (dir = {:?})",
    crate::asset::config::ASSETS_CONFIG_FILENAME_JSON,
    crate::asset::config::ASSETS_CONFIG_FILENAME_JSON5,
    .0.display())]
    MultipleConfigurationFiles(PathBuf),
}
