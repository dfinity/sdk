use crate::error::fs::FsError;
use crate::error::identity::IdentityError;
use crate::error::keyring::KeyringError;
use crate::error::wallet_config::WalletConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RemoveIdentityError {
    #[error("Cannot delete the anonymous identity.")]
    CannotDeleteAnonymousIdentity(),

    #[error("Cannot delete the default identity.")]
    CannotDeleteDefaultIdentity(),

    #[error("Failed to display linked wallets: {0}")]
    DisplayLinkedWalletsFailed(WalletConfigError),

    #[error("If you want to remove an identity with configured wallets, please use the --drop-wallets flag.")]
    DropWalletsFlagRequiredToRemoveIdentityWithWallets(),

    #[error("Failed to remove identity directory: {0}")]
    RemoveIdentityDirectoryFailed(FsError),

    #[error("Failed to remove identity file: {0}")]
    RemoveIdentityFileFailed(FsError),

    #[error("Failed to remove identity from keyring: {0}")]
    RemoveIdentityFromKeyringFailed(KeyringError),

    #[error("Identity must exist: {0}")]
    RequireIdentityExistsFailed(IdentityError),
}
