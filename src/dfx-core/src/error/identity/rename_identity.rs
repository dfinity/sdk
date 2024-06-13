use crate::error::fs::FsError;
use crate::error::identity::get_identity_config_or_default::GetIdentityConfigOrDefaultError;
use crate::error::identity::load_pem::LoadPemError;
use crate::error::identity::map_wallets_to_renamed_identity::MapWalletsToRenamedIdentityError;
use crate::error::identity::require_identity_exists::RequireIdentityExistsError;
use crate::error::identity::save_identity_configuration::SaveIdentityConfigurationError;
use crate::error::identity::save_pem::SavePemError;
use crate::error::identity::write_default_identity::WriteDefaultIdentityError;
use crate::error::keyring::KeyringError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenameIdentityError {
    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Failed to get identity config")]
    GetIdentityConfigFailed(#[source] GetIdentityConfigOrDefaultError),

    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Identity does not exist")]
    IdentityDoesNotExist(#[source] RequireIdentityExistsError),

    #[error("Failed to load pem")]
    LoadPemFailed(#[source] LoadPemError),

    #[error("Failed to map wallets to renamed identity")]
    MapWalletsToRenamedIdentityFailed(#[source] MapWalletsToRenamedIdentityError),

    #[error("Failed to remove identity from keyring")]
    RemoveIdentityFromKeyringFailed(#[source] KeyringError),

    #[error("Cannot rename identity directory")]
    RenameIdentityDirectoryFailed(#[source] FsError),

    #[error("Failed to save identity configuration")]
    SaveIdentityConfigurationFailed(#[source] SaveIdentityConfigurationError),

    #[error("Failed to save pem")]
    SavePemFailed(#[source] SavePemError),

    #[error("Failed to switch over default identity settings")]
    SwitchDefaultIdentitySettingsFailed(#[source] WriteDefaultIdentityError),
}
