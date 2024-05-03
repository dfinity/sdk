use crate::error::config::ConfigError;
use crate::error::identity::rename_wallet_global_config_key::RenameWalletGlobalConfigKeyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MapWalletsToRenamedIdentityError {
    #[error("Failed to get config directory for identity manager")]
    GetConfigDirectoryFailed(#[source] ConfigError),

    #[error("Failed to get shared network data directory")]
    GetSharedNetworkDataDirectoryFailed(#[source] ConfigError),

    #[error("Failed to rename wallet global config key")]
    RenameWalletGlobalConfigKeyFailed(#[source] RenameWalletGlobalConfigKeyError),
}
