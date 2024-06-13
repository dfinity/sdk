use crate::error::fs::FsError;
use crate::error::identity::require_identity_exists::RequireIdentityExistsError;
use crate::error::keyring::KeyringError;
use crate::error::wallet_config::WalletConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RemoveIdentityError {
    #[error("Cannot delete the anonymous identity.")]
    CannotDeleteAnonymousIdentity(),

    #[error("Cannot delete the default identity.")]
    CannotDeleteDefaultIdentity(),

    #[error("Failed to display linked wallets")]
    DisplayLinkedWalletsFailed(#[source] WalletConfigError),

    #[error("If you want to remove an identity with configured wallets, please use the --drop-wallets flag.")]
    DropWalletsFlagRequiredToRemoveIdentityWithWallets(),

    #[error("Failed to remove identity directory")]
    RemoveIdentityDirectoryFailed(#[source] FsError),

    #[error("Failed to remove identity file")]
    RemoveIdentityFileFailed(#[source] FsError),

    #[error("Failed to remove identity from keyring")]
    RemoveIdentityFromKeyringFailed(#[source] KeyringError),

    #[error("Identity must exist")]
    RequireIdentityExistsFailed(#[source] RequireIdentityExistsError),
}
