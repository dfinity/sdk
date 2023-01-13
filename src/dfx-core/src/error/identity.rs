use crate::error::encryption::EncryptionError;
use crate::error::foundation::FoundationError;
use crate::error::io::IoError;
use crate::error::keyring::KeyringError;
use crate::error::structured_file::StructuredFileError;
use crate::error::wallet_config::WalletConfigError;

use ic_agent::identity::PemError;

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Cannot delete the default identity.")]
    CannotDeleteDefaultIdentity(),

    #[error("Cannot delete the anonymous identity.")]
    CannotDeleteAnonymousIdentity(),

    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Cannot create identity directory: {0}")]
    CreateIdentityDirectoryFailed(IoError),

    #[error("Failed to derive extended secret key from path: {0}")]
    DeriveExtendedKeyFromPathFailed(bip32::Error),

    #[error("Failed to display linked wallets: {0}")]
    DisplayLinkedWalletsFailed(WalletConfigError),

    #[error("If you want to remove an identity with configured wallets, please use the --drop-wallets flag.")]
    DropWalletsFlagRequiredToRemoveIdentityWithWallets(),

    #[error("Cannot encrypt PEM file: {0}")]
    EncryptPemFileFailed(PathBuf, EncryptionError),

    #[error("Failed to generate a fresh secp256k1 key: {0}")]
    GenerateFreshSecp256k1KeyFailed(Box<sec1::Error>),

    #[error("Failed to get legacy pem path: {0}")]
    GetLegacyPemPathFailed(FoundationError),

    #[error("Identity already exists.")]
    IdentityAlreadyExists(),

    #[error("Identity {0} does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("Failed to load configuration for identity '{0}': {1}")]
    LoadIdentityConfigurationFailed(String, StructuredFileError),

    #[error("Failed to load identity manager configuration: {0}")]
    LoadIdentityManagerConfigurationFailed(StructuredFileError),

    #[error("Failed to migrate legacy identity")]
    MigrateLegacyIdentityFailed(IoError),

    #[error("Cannot read identity file '{0}': {1:#}")]
    ReadIdentityFileFailed(String, Box<PemError>),

    #[error("Failed to remove identity directory: {0}")]
    RemoveIdentityDirectoryFailed(IoError),

    #[error("Failed to remove identity from keyring: {0}")]
    RemoveIdentityFromKeyringFailed(KeyringError),

    #[error("Failed to remove identity file: {0}")]
    RemoveIdentityFileFailed(IoError),

    #[error("Cannot rename identity directory: {0}")]
    RenameIdentityDirectoryFailed(IoError),

    #[error("An Identity named {0} cannot be created as it is reserved for internal use.")]
    ReservedIdentityName(String),

    #[error("Failed to save identity manager configuration: {0}")]
    SaveIdentityManagerConfigurationFailed(StructuredFileError),

    #[error("Cannot write PEM file: {0}")]
    WritePemFileFailed(IoError),
}
