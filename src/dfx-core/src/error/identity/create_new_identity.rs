use crate::error::fs::FsError;
use crate::error::identity::convert_mnemonic_to_key::ConvertMnemonicToKeyError;
use crate::error::identity::generate_key::GenerateKeyError;
use crate::error::identity::load_pem_from_file::LoadPemFromFileError;
use crate::error::identity::remove_identity::RemoveIdentityError;
use crate::error::identity::save_identity_configuration::SaveIdentityConfigurationError;
use crate::error::identity::save_pem::SavePemError;
use crate::error::identity::IdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateNewIdentityError {
    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Failed to clean up previous creation attempts: {0}")]
    CleanupPreviousCreationAttemptsFailed(FsError),

    #[error("Failed to create identity config: {0}")]
    ConvertMnemonicToKeyFailed(ConvertMnemonicToKeyError),

    #[error("Convert secret key to sec1 Pem failed: {0}")]
    ConvertSecretKeyToSec1PemFailed(Box<sec1::Error>),

    #[error("Failed to create identity config: {0}")]
    CreateIdentityConfigFailed(IdentityError),

    #[error("Failed to create mnemonic from phrase: {0}")]
    CreateMnemonicFromPhraseFailed(String),

    #[error("Failed to create temporary identity directory: {0}")]
    CreateTemporaryIdentityDirectoryFailed(FsError),

    #[error("Failed to generate key: {0}")]
    GenerateKeyFailed(GenerateKeyError),

    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Failed to load pem file: {0}")]
    LoadPemFromFileFailed(LoadPemFromFileError),

    #[error("Failed to remove identity: {0}")]
    RemoveIdentityFailed(RemoveIdentityError),

    #[error("Failed to rename temporary directory to permanent identity directory: {0}")]
    RenameTemporaryIdentityDirectoryFailed(FsError),

    #[error("Failed to save identity configuration: {0}")]
    SaveIdentityConfigurationFailed(SaveIdentityConfigurationError),

    #[error("Failed to save pem: {0}")]
    SavePemFailed(SavePemError),

    #[error("Failed to switch back over to the identity you're replacing: {0}")]
    SwitchBackToIdentityFailed(IdentityError),

    #[error("Failed to temporarily switch over to anonymous identity: {0}")]
    SwitchToAnonymousIdentityFailed(IdentityError),

    #[error("Failed to validate pem file: {0}")]
    ValidatePemFileFailed(IdentityError),
}
