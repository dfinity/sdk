use crate::error::fs::FsError;
use crate::error::identity::convert_mnemonic_to_key::ConvertMnemonicToKeyError;
use crate::error::identity::create_identity_config::CreateIdentityConfigError;
use crate::error::identity::generate_key::GenerateKeyError;
use crate::error::identity::load_pem_from_file::LoadPemFromFileError;
use crate::error::identity::remove_identity::RemoveIdentityError;
use crate::error::identity::save_identity_configuration::SaveIdentityConfigurationError;
use crate::error::identity::save_pem::SavePemError;
use crate::error::identity::use_identity_by_name::UseIdentityByNameError;
use crate::error::identity::validate_pem_file::ValidatePemFileError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateNewIdentityError {
    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Failed to clean up previous creation attempts")]
    CleanupPreviousCreationAttemptsFailed(#[source] FsError),

    #[error("Failed to create identity config")]
    ConvertMnemonicToKeyFailed(#[source] ConvertMnemonicToKeyError),

    #[error("Convert secret key to sec1 Pem failed")]
    ConvertSecretKeyToSec1PemFailed(#[source] Box<sec1::Error>),

    #[error("Failed to create identity config")]
    CreateIdentityConfigFailed(#[source] CreateIdentityConfigError),

    #[error("Failed to create mnemonic from phrase: {0}")]
    CreateMnemonicFromPhraseFailed(String),

    #[error("Failed to create temporary identity directory")]
    CreateTemporaryIdentityDirectoryFailed(#[source] FsError),

    #[error("Failed to generate key")]
    GenerateKeyFailed(#[source] GenerateKeyError),

    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Failed to load pem file")]
    LoadPemFromFileFailed(#[source] LoadPemFromFileError),

    #[error("Failed to remove identity")]
    RemoveIdentityFailed(#[source] RemoveIdentityError),

    #[error("Failed to rename temporary directory to permanent identity directory")]
    RenameTemporaryIdentityDirectoryFailed(#[source] FsError),

    #[error("Failed to save identity configuration")]
    SaveIdentityConfigurationFailed(#[source] SaveIdentityConfigurationError),

    #[error("Failed to save pem")]
    SavePemFailed(#[source] SavePemError),

    #[error("Failed to switch back over to the identity you're replacing")]
    SwitchBackToIdentityFailed(#[source] UseIdentityByNameError),

    #[error("Failed to temporarily switch over to anonymous identity")]
    SwitchToAnonymousIdentityFailed(#[source] UseIdentityByNameError),

    #[error("Failed to validate pem file")]
    ValidatePemFileFailed(#[source] ValidatePemFileError),
}
