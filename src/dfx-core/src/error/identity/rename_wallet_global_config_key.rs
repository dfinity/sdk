use crate::error::wallet_config::WalletConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenameWalletGlobalConfigKeyError {
    #[error("Failed to rename '{0}' to '{1}' in the global wallet config")]
    RenameWalletFailed(Box<String>, Box<String>, #[source] WalletConfigError),
}
