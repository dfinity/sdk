use crate::error::get_asset_config::GetAssetConfigError;
use crate::error::load_config::AssetLoadConfigError;
use dfx_core::error::fs::CanonicalizePathError;
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
    #[error("invalid directory entry")]
    InvalidDirectoryEntry(#[source] CanonicalizePathError),

    /// Failed to canonicalize a source directory.
    #[error("invalid source directory")]
    InvalidSourceDirectory(#[source] CanonicalizePathError),

    /// Failed to load the asset configuration for a directory.
    #[error("Failed to load asset configuration: {0}")]
    LoadConfigFailed(AssetLoadConfigError),

    /// One or more assets use the hardened security policy but don't actually specify any hardenings compared to the standard security policy.
    #[error("This project uses the hardened security policy for some assets, but does not actually configure any custom improvements over the standard policy. To get started, look at 'dfx info canister-security-policy'. It shows the default security policy along with suggestions on how to improve it.\n{0}")]
    HardenedSecurityPolicyIsNotHardened(String),
}
