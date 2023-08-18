use crate::error::fs::FsError;
use crate::error::identity::get_identity_config_or_default::GetIdentityConfigOrDefaultError;
use crate::error::identity::load_pem::LoadPemError;
use crate::error::identity::save_identity_configuration::SaveIdentityConfigurationError;
use crate::error::identity::save_pem::SavePemError;
use crate::error::identity::IdentityError;
use crate::error::keyring::KeyringError;
use thiserror::Error;
use crate::error::identity::map_wallets_to_renamed_identity::MapWalletsToRenamedIdentityError;

#[derive(Error, Debug)]
pub enum RenameIdentityError {
    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Failed to get identity config: {0}")]
    GetIdentityConfigFailed(GetIdentityConfigOrDefaultError),

    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Identity does not exist: {0}")]
    IdentityDoesNotExist(IdentityError),

    #[error("Failed to load pem: {0}")]
    LoadPemFailed(LoadPemError),

    #[error("Failed to map wallets to renamed identity: {0}")]
    MapWalletsToRenamedIdentityFailed(MapWalletsToRenamedIdentityError),

    #[error("Failed to remove identity from keyring: {0}")]
    RemoveIdentityFromKeyringFailed(KeyringError),

    #[error("Cannot rename identity directory: {0}")]
    RenameIdentityDirectoryFailed(FsError),

    #[error("Failed to save identity configuration: {0}")]
    SaveIdentityConfigurationFailed(SaveIdentityConfigurationError),

    #[error("Failed to save pem: {0}")]
    SavePemFailed(SavePemError),

    #[error("Failed to switch over default identity settings: {0}")]
    SwitchDefaultIdentitySettingsFailed(IdentityError),
}
