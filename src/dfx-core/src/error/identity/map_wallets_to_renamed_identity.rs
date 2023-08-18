use crate::error::config::ConfigError;
use crate::error::identity::rename_wallet_global_config_key::RenameWalletGlobalConfigKeyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MapWalletsToRenamedIdentityError {
    #[error("Failed to get config directory for identity manager: {0}")]
    GetConfigDirectoryFailed(ConfigError),

    #[error("Failed to get shared network data directory: {0}")]
    GetSharedNetworkDataDirectoryFailed(ConfigError),

    #[error("Failed to rename wallet global config key: {0}")]
    RenameWalletGlobalConfigKeyFailed(RenameWalletGlobalConfigKeyError),
}
