use crate::error::fs::FsError;
use crate::error::identity::IdentityError;
use crate::error::keyring::KeyringError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenameIdentityError {
    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Failed to get identity config: {0}")]
    GetIdentityConfigFailed(IdentityError),

    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Identity does not exist: {0}")]
    IdentityDoesNotExist(IdentityError),

    #[error("Failed to load pem: {0}")]
    LoadPemFailed(IdentityError),

    #[error("Failed to map wallets to renamed identity: {0}")]
    MapWalletsToRenamedIdentityFailed(IdentityError /*MapWalletsToRenamedIdentityError*/),

    #[error("Failed to remove identity from keyring: {0}")]
    RemoveIdentityFromKeyringFailed(KeyringError),

    #[error("Cannot rename identity directory: {0}")]
    RenameIdentityDirectoryFailed(FsError),

    #[error("Failed to save identity configuration: {0}")]
    SaveIdentityConfigurationFailed(IdentityError),

    #[error("Failed to save pem: {0}")]
    SavePemFailed(IdentityError),

    #[error("Failed to switch over default identity settings: {0}")]
    SwitchDefaultIdentitySettingsFailed(IdentityError),
}
