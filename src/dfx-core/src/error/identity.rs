use crate::error::fs::{
    ReadFileError, ReadPermissionsError, RemoveDirectoryAndContentsError, RemoveDirectoryError,
    SetPermissionsError, WriteFileError,
};
use crate::error::{
    config::ConfigError,
    encryption::EncryptionError,
    fs::{
        CopyFileError, CreateDirAllError, EnsureParentDirExistsError, FsError, NoParentPathError,
    },
    get_user_home::GetUserHomeError,
    keyring::KeyringError,
    structured_file::StructuredFileError,
    wallet_config::{SaveWalletConfigError, WalletConfigError},
};
use candid::types::principal::PrincipalError;
use ic_agent::identity::PemError;
use ic_identity_hsm::HardwareIdentityError;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CallSenderFromWalletError {
    #[error("Failed to read principal from id '{0}', and did not find a wallet for that identity")]
    ParsePrincipalFromIdFailedAndNoWallet(String, #[source] PrincipalError),

    #[error("Failed to read principal from id '{0}' ({1}), and failed to load the wallet for that identity"
    )]
    ParsePrincipalFromIdFailedAndGetWalletCanisterIdFailed(
        String,
        PrincipalError,
        #[source] WalletConfigError,
    ),
}

#[derive(Error, Debug)]
pub enum ConvertMnemonicToKeyError {
    #[error("Failed to derive extended secret key from path")]
    DeriveExtendedKeyFromPathFailed(#[source] bip32::Error),
}

#[derive(Error, Debug)]
pub enum CreateIdentityConfigError {
    #[error("Failed to generate a fresh encryption configuration")]
    GenerateFreshEncryptionConfigurationFailed(#[source] EncryptionError),
}

#[derive(Error, Debug)]
pub enum CreateNewIdentityError {
    #[error("Cannot create an anonymous identity.")]
    CannotCreateAnonymousIdentity(),

    #[error("Failed to clean up previous creation attempts")]
    CleanupPreviousCreationAttemptsFailed(#[from] RemoveDirectoryAndContentsError),

    #[error("Failed to create identity config")]
    ConvertMnemonicToKeyFailed(#[source] ConvertMnemonicToKeyError),

    #[error("Convert secret key to sec1 Pem failed")]
    ConvertSecretKeyToSec1PemFailed(#[source] Box<sec1::Error>),

    #[error("Failed to create identity config")]
    CreateIdentityConfigFailed(#[source] CreateIdentityConfigError),

    #[error("Failed to create mnemonic from phrase: {0}")]
    CreateMnemonicFromPhraseFailed(String),

    #[error("failed to create temporary identity directory")]
    CreateTemporaryIdentityDirectoryFailed(#[source] CreateDirAllError),

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

#[derive(Error, Debug)]
pub enum ExportIdentityError {
    #[error("Failed to get identity config")]
    GetIdentityConfigFailed(#[source] GetIdentityConfigOrDefaultError),

    #[error("The specified identity does not exist")]
    IdentityDoesNotExist(#[source] RequireIdentityExistsError),

    #[error("Failed to load pem file")]
    LoadPemFailed(#[source] LoadPemError),

    #[error("Could not translate pem file to text")]
    TranslatePemContentToTextFailed(#[source] FromUtf8Error),

    #[error("Failed to validate pem file")]
    ValidatePemFileFailed(#[source] ValidatePemFileError),
}

#[derive(Error, Debug)]
pub enum GenerateKeyError {
    #[error("Failed to convert mnemonic to key")]
    ConvertMnemonicToKeyFailed(#[source] ConvertMnemonicToKeyError),

    #[error("Failed to generate a fresh secp256k1 key")]
    GenerateFreshSecp256k1KeyFailed(#[source] Box<sec1::Error>),
}

#[derive(Error, Debug)]
pub enum GetIdentityConfigOrDefaultError {
    #[error("Failed to load configuration for identity '{0}'")]
    LoadIdentityConfigurationFailed(String, #[source] StructuredFileError),
}

#[derive(Error, Debug)]
pub enum GetLegacyCredentialsPemPathError {
    #[error("Failed to get legacy pem path")]
    GetLegacyPemPathFailed(#[source] GetUserHomeError),
}

#[derive(Error, Debug)]
pub enum InitializeIdentityManagerError {
    #[error("Cannot create identity directory")]
    CreateIdentityDirectoryFailed(#[source] CreateDirAllError),

    #[error("Failed to generate key")]
    GenerateKeyFailed(#[source] GenerateKeyError),

    #[error(transparent)]
    GetLegacyCredentialsPemPathFailed(#[from] GetLegacyCredentialsPemPathError),

    #[error("failed to migrate legacy identity")]
    MigrateLegacyIdentityFailed(#[source] CopyFileError),

    #[error("Failed to save configuration")]
    SaveConfigurationFailed(#[source] StructuredFileError),

    #[error("Failed to write pem to file")]
    WritePemToFileFailed(#[source] WritePemToFileError),
}

#[derive(Error, Debug)]
pub enum InstantiateIdentityFromNameError {
    #[error("Failed to get principal of identity: {0}")]
    GetIdentityPrincipalFailed(String),

    #[error("Failed to load identity")]
    LoadIdentityFailed(#[source] LoadIdentityError),

    #[error("Identity must exist")]
    RequireIdentityExistsFailed(#[source] RequireIdentityExistsError),
}

#[derive(Error, Debug)]
pub enum LoadIdentityError {
    #[error("Failed to get identity config")]
    GetIdentityConfigOrDefaultFailed(#[source] GetIdentityConfigOrDefaultError),

    #[error("Failed to instantiate identity")]
    NewIdentityFailed(#[source] NewIdentityError),
}

#[derive(Error, Debug)]
pub enum LoadPemError {
    #[error("Failed to load PEM file from file")]
    LoadFromFileFailed(#[source] LoadPemFromFileError),

    #[error("Failed to load PEM file from keyring for identity '{0}'")]
    LoadFromKeyringFailed(Box<String>, #[source] KeyringError),
}

#[derive(Error, Debug)]
pub enum LoadPemFromFileError {
    #[error("Failed to decrypt PEM file at {0}")]
    DecryptPemFileFailed(PathBuf, #[source] EncryptionError),

    #[error("failed to read pem file")]
    ReadPemFileFailed(#[from] ReadFileError),
}

#[derive(Error, Debug)]
pub enum LoadPemIdentityError {
    #[error("Cannot read identity file '{0}'")]
    ReadIdentityFileFailed(String, #[source] Box<PemError>),
}

#[derive(Error, Debug)]
pub enum MapWalletsToRenamedIdentityError {
    #[error("Failed to get config directory for identity manager")]
    GetConfigDirectoryFailed(#[source] ConfigError),

    #[error("Failed to get shared network data directory")]
    GetSharedNetworkDataDirectoryFailed(#[source] ConfigError),

    #[error("Failed to rename wallet global config key")]
    RenameWalletGlobalConfigKeyFailed(#[source] RenameWalletGlobalConfigKeyError),
}

#[derive(Error, Debug)]
pub enum NewHardwareIdentityError {
    #[error("Failed to instantiate hardware identity for identity '{0}'")]
    InstantiateHardwareIdentityFailed(String, #[source] Box<HardwareIdentityError>),
}

#[derive(Error, Debug)]
pub enum NewIdentityError {
    #[error("Failed to load PEM")]
    LoadPemFailed(#[source] LoadPemError),

    #[error("Failed to load PEM identity")]
    LoadPemIdentityFailed(#[source] LoadPemIdentityError),

    #[error("Failed to instantiate hardware identity")]
    NewHardwareIdentityFailed(#[source] NewHardwareIdentityError),
}

#[derive(Error, Debug)]
pub enum NewIdentityManagerError {
    #[error("Failed to get config directory for identity manager")]
    GetConfigDirectoryFailed(#[source] ConfigError),

    #[error("Failed to load identity manager configuration")]
    LoadIdentityManagerConfigurationFailed(#[source] StructuredFileError),

    #[error("Failed to initialize identity manager")]
    InitializeFailed(#[source] InitializeIdentityManagerError),

    #[error("The specified identity must exist")]
    OverrideIdentityMustExist(#[source] RequireIdentityExistsError),

    #[error(r#"No identity configuration found.  Please run "dfx identity get-principal" or "dfx identity new <identity name>" to create a new identity."#)]
    NoIdentityConfigurationFound,
}

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

    #[error("failed to remove identity directory")]
    RemoveIdentityDirectoryFailed(#[source] RemoveDirectoryError),

    #[error("Failed to remove identity file")]
    RemoveIdentityFileFailed(#[source] FsError),

    #[error("Failed to remove identity from keyring")]
    RemoveIdentityFromKeyringFailed(#[source] KeyringError),

    #[error("Identity must exist")]
    RequireIdentityExistsFailed(#[source] RequireIdentityExistsError),
}

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

#[derive(Error, Debug)]
pub enum RenameWalletGlobalConfigKeyError {
    #[error("Failed to rename '{0}' to '{1}' in the global wallet config")]
    RenameWalletFailed(Box<String>, Box<String>, #[source] WalletConfigError),

    #[error(transparent)]
    SaveWalletConfig(#[from] SaveWalletConfigError),
}

#[derive(Error, Debug)]
pub enum RequireIdentityExistsError {
    #[error("Identity {0} does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("An Identity named {0} cannot be created as it is reserved for internal use.")]
    ReservedIdentityName(String),
}

#[derive(Error, Debug)]
pub enum SaveIdentityConfigurationError {
    #[error("failed to ensure identity configuration directory exists")]
    EnsureIdentityConfigurationDirExistsFailed(#[source] EnsureParentDirExistsError),

    #[error("Failed to save identity configuration")]
    SaveIdentityConfigurationFailed(#[source] StructuredFileError),
}

#[derive(Error, Debug)]
pub enum SavePemError {
    #[error("Cannot save PEM content for an HSM.")]
    CannotSavePemContentForHsm(),

    #[error("Failed to write PEM to file")]
    WritePemToFileFailed(#[source] WritePemToFileError),

    #[error("Failed to write PEM to keyring")]
    WritePemToKeyringFailed(#[source] KeyringError),
}

#[derive(Error, Debug)]
pub enum UseIdentityByNameError {
    #[error("Identity must exist")]
    RequireIdentityExistsFailed(#[source] RequireIdentityExistsError),

    #[error("Failed to write default identity")]
    WriteDefaultIdentityFailed(#[source] WriteDefaultIdentityError),
}

#[derive(Error, Debug)]
pub enum ValidatePemFileError {
    #[error(transparent)]
    PemError(#[from] ic_agent::identity::PemError),

    #[error(
        "Ed25519 v1 keys (those generated by OpenSSL) are not supported. Try again with a v2 key"
    )]
    UnsupportedKeyVersion(),

    #[error("Failed to validate PEM content")]
    ValidatePemContentFailed(#[source] Box<PemError>),
}

#[derive(Error, Debug)]
pub enum WriteDefaultIdentityError {
    #[error("Failed to save identity manager configuration")]
    SaveIdentityManagerConfigurationFailed(#[source] StructuredFileError),
}

#[derive(Error, Debug)]
pub enum WritePemContentError {
    #[error(transparent)]
    CreateDirAll(#[from] CreateDirAllError),

    #[error(transparent)]
    NoParent(#[from] NoParentPathError),

    #[error(transparent)]
    ReadPermissions(#[from] ReadPermissionsError),

    #[error(transparent)]
    SetPermissions(#[from] SetPermissionsError),

    #[error(transparent)]
    Write(#[from] WriteFileError),
}

#[derive(Error, Debug)]
pub enum WritePemToFileError {
    #[error("Failed to encrypt PEM file")]
    EncryptPemFileFailed(PathBuf, #[source] EncryptionError),

    #[error("failed to write PEM content")]
    WritePemContentFailed(#[source] WritePemContentError),
}
