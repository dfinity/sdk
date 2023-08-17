use super::pem_utils::validate_pem_file;
use super::{keyring_mock, WALLET_CONFIG_FILENAME};
use crate::config::directories::get_config_dfx_dir_path;
use crate::error::encryption::EncryptionError;
use crate::error::encryption::EncryptionError::{NonceGenerationFailed, SaltGenerationFailed};
use crate::error::fs::FsError;
use crate::error::identity::export_identity::ExportIdentityError;
use crate::error::identity::export_identity::ExportIdentityError::TranslatePemContentToTextFailed;
use crate::error::identity::get_legacy_credentials_pem_path::GetLegacyCredentialsPemPathError;
use crate::error::identity::get_legacy_credentials_pem_path::GetLegacyCredentialsPemPathError::GetLegacyPemPathFailed;
use crate::error::identity::initialize_identity_manager::InitializeIdentityManagerError;
use crate::error::identity::initialize_identity_manager::InitializeIdentityManagerError::{
    CreateIdentityDirectoryFailed, GenerateKeyFailed, MigrateLegacyIdentityFailed,
    WritePemToFileFailed,
};
use crate::error::identity::new_identity_manager::NewIdentityManagerError;
use crate::error::identity::new_identity_manager::NewIdentityManagerError::LoadIdentityManagerConfigurationFailed;
use crate::error::identity::rename_identity::RenameIdentityError;
use crate::error::identity::rename_identity::RenameIdentityError::{
    GetIdentityConfigFailed, LoadPemFailed, MapWalletsToRenamedIdentityFailed,
    RenameIdentityDirectoryFailed, SavePemFailed, SwitchDefaultIdentitySettingsFailed,
};
use crate::error::identity::IdentityError;
use crate::error::identity::IdentityError::{
    CleanupPreviousCreationAttemptsFailed, ConvertSecretKeyToSec1PemFailed,
    CreateMnemonicFromPhraseFailed, CreateTemporaryIdentityDirectoryFailed,
    DisplayLinkedWalletsFailed, DropWalletsFlagRequiredToRemoveIdentityWithWallets,
    EnsureIdentityConfigurationDirExistsFailed, GenerateFreshEncryptionConfigurationFailed,
    GetIdentityPrincipalFailed, IdentityAlreadyExists, LoadIdentityConfigurationFailed,
    RemoveIdentityDirectoryFailed, RemoveIdentityFileFailed, RemoveIdentityFromKeyringFailed,
    RenameTemporaryIdentityDirectoryFailed, SaveIdentityConfigurationFailed,
    SwitchBackToIdentityFailed, SwitchToAnonymousIdentityFailed,
};
use crate::error::structured_file::StructuredFileError;
use crate::foundation::get_user_home;
use crate::fs::composite::ensure_parent_dir_exists;
use crate::identity::identity_file_locations::{IdentityFileLocations, IDENTITY_PEM};
use crate::identity::identity_manager::IdentityStorageModeError::UnknownStorageMode;
use crate::identity::{
    pem_safekeeping, pem_utils, Identity as DfxIdentity, ANONYMOUS_IDENTITY_NAME, IDENTITY_JSON,
    TEMP_IDENTITY_PREFIX,
};
use crate::json::{load_json_file, save_json_file};
use bip32::XPrv;
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use candid::Principal;
use k256::pkcs8::LineEnding;
use k256::SecretKey;
use ring::{rand, rand::SecureRandom};
use sec1::EncodeEcPrivateKey;
use serde::{Deserialize, Serialize};
use slog::{debug, trace, Logger};
use std::boxed::Box;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

const DEFAULT_IDENTITY_NAME: &str = "default";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Configuration {
    #[serde(default = "default_identity")]
    pub default: String,
}

fn default_identity() -> String {
    String::from(DEFAULT_IDENTITY_NAME)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IdentityConfiguration {
    pub hsm: Option<HardwareIdentityConfiguration>,

    /// If the identity's PEM file is encrypted on disk this contains everything (except the password) to decrypt the file.
    pub encryption: Option<EncryptionConfiguration>,

    /// If the identity's PEM file is stored in the system's keyring, this field contains the identity's name WITHOUT the common prefix.
    pub keyring_identity_suffix: Option<String>,
}

/// The information necessary to de- and encrypt (except the password) the identity's .pem file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionConfiguration {
    /// Salt used for deriving the key from the password
    pub pw_salt: String,

    /// 96 bit Nonce used to decrypt the file
    pub file_nonce: Vec<u8>,
}

impl EncryptionConfiguration {
    /// Generates a random salt and nonce. Use this for every new identity
    pub fn new() -> Result<Self, EncryptionError> {
        let mut nonce: [u8; 12] = [0; 12];
        let mut salt: [u8; 32] = [0; 32];
        let sr = rand::SystemRandom::new();
        sr.fill(&mut nonce).map_err(NonceGenerationFailed)?;
        sr.fill(&mut salt).map_err(SaltGenerationFailed)?;

        let pw_salt = hex::encode(salt);
        let file_nonce = nonce.into();

        Ok(Self {
            pw_salt,
            file_nonce,
        })
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HardwareIdentityConfiguration {
    #[cfg_attr(
        not(windows),
        doc = r#"The file path to the opensc-pkcs11 library e.g. "/usr/local/lib/opensc-pkcs11.so""#
    )]
    #[cfg_attr(
        windows,
        doc = r#"The file path to the opensc-pkcs11 library e.g. "C:\Program Files (x86)\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll"#
    )]
    pub pkcs11_lib_path: String,

    /// A sequence of pairs of hex digits
    pub key_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq, Eq)]
pub enum IdentityStorageMode {
    Keyring,
    PasswordProtected,
    Plaintext,
}

#[derive(Error, Debug)]
pub enum IdentityStorageModeError {
    #[error("Unknown storage mode: {0}")]
    UnknownStorageMode(String),
}

impl FromStr for IdentityStorageMode {
    type Err = IdentityStorageModeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "keyring" => Ok(IdentityStorageMode::Keyring),
            "password-protected" => Ok(IdentityStorageMode::PasswordProtected),
            "plaintext" => Ok(IdentityStorageMode::Plaintext),
            other => Err(UnknownStorageMode(other.to_string())),
        }
    }
}

impl Default for IdentityStorageMode {
    fn default() -> Self {
        Self::Keyring
    }
}

pub enum IdentityCreationParameters {
    Pem {
        mode: IdentityStorageMode,
    },
    PemFile {
        src_pem_file: PathBuf,
        mode: IdentityStorageMode,
    },
    SeedPhrase {
        mnemonic: String,
        mode: IdentityStorageMode,
    },
    Hardware {
        hsm: HardwareIdentityConfiguration,
    },
}

#[derive(Clone, Debug)]
pub struct IdentityManager {
    identity_json_path: PathBuf,
    file_locations: IdentityFileLocations,
    configuration: Configuration,
    selected_identity: String,
    selected_identity_principal: Option<Principal>,
}

impl IdentityManager {
    pub fn new(
        logger: &Logger,
        identity_override: &Option<String>,
    ) -> Result<Self, NewIdentityManagerError> {
        let config_dfx_dir_path =
            get_config_dfx_dir_path().map_err(NewIdentityManagerError::GetConfigDirectoryFailed)?;
        let identity_root_path = config_dfx_dir_path.join("identity");
        let identity_json_path = config_dfx_dir_path.join("identity.json");

        let configuration = if identity_json_path.exists() {
            load_configuration(&identity_json_path)
                .map_err(LoadIdentityManagerConfigurationFailed)?
        } else {
            initialize(logger, &identity_json_path, &identity_root_path)
                .map_err(NewIdentityManagerError::InitializeFailed)?
        };

        let selected_identity = identity_override
            .clone()
            .unwrap_or_else(|| configuration.default.clone());
        let file_locations = IdentityFileLocations::new(identity_root_path);

        let mgr = IdentityManager {
            identity_json_path,
            file_locations,
            configuration,
            selected_identity,
            selected_identity_principal: None,
        };

        if let Some(identity) = identity_override {
            mgr.require_identity_exists(logger, identity)
                .map_err(NewIdentityManagerError::OverrideIdentityMustExist)?;
        }

        Ok(mgr)
    }

    pub fn get_selected_identity_principal(&self) -> Option<Principal> {
        self.selected_identity_principal
    }

    /// Create an Identity instance for use with an Agent
    pub fn instantiate_selected_identity(
        &mut self,
        log: &Logger,
    ) -> Result<Box<DfxIdentity>, IdentityError> {
        let name = self.selected_identity.clone();
        self.instantiate_identity_from_name(name.as_str(), log)
    }

    /// Provide a valid Identity name and create its Identity instance for use with an Agent
    pub fn instantiate_identity_from_name(
        &mut self,
        identity_name: &str,
        log: &Logger,
    ) -> Result<Box<DfxIdentity>, IdentityError> {
        let identity = match identity_name {
            ANONYMOUS_IDENTITY_NAME => Box::new(DfxIdentity::anonymous()),
            identity_name => {
                self.require_identity_exists(log, identity_name)?;
                Box::new(self.load_identity(identity_name, log)?)
            }
        };
        use ic_agent::identity::Identity;
        self.selected_identity_principal =
            Some(identity.sender().map_err(GetIdentityPrincipalFailed)?);
        Ok(identity)
    }

    fn load_identity(&self, name: &str, log: &Logger) -> Result<DfxIdentity, IdentityError> {
        let config = self.get_identity_config_or_default(name)?;
        DfxIdentity::new(name, config, self.file_locations(), log)
    }

    /// Create a new identity (name -> generated key)
    ///
    /// `force`: If the identity already exists, remove and re-create it.
    pub fn create_new_identity(
        &mut self,
        log: &Logger,
        name: &str,
        parameters: IdentityCreationParameters,
        force: bool,
    ) -> Result<(), IdentityError> {
        if name == ANONYMOUS_IDENTITY_NAME {
            return Err(IdentityError::CannotCreateAnonymousIdentity());
        }

        trace!(log, "Creating identity '{name}'.");
        let identity_in_use = self.get_selected_identity_name().clone();
        // cannot delete an identity in use. Use anonymous identity temporarily if we force-overwrite the identity currently in use
        let temporarily_use_anonymous_identity = identity_in_use == name && force;

        if self.require_identity_exists(log, name).is_ok() {
            trace!(log, "Identity already exists.");
            if force {
                if temporarily_use_anonymous_identity {
                    self.use_identity_named(log, ANONYMOUS_IDENTITY_NAME)
                        .map_err(|e| SwitchToAnonymousIdentityFailed(Box::new(e)))?;
                }
                self.remove(log, name, true, None)?;
            } else {
                return Err(IdentityAlreadyExists());
            }
        }

        fn create_identity_config(
            log: &Logger,
            mode: IdentityStorageMode,
            name: &str,
            hardware_config: Option<HardwareIdentityConfiguration>,
        ) -> Result<IdentityConfiguration, IdentityError> {
            if let Some(hsm) = hardware_config {
                Ok(IdentityConfiguration {
                    hsm: Some(hsm),
                    ..Default::default()
                })
            } else {
                match mode {
                    IdentityStorageMode::Keyring => {
                        if keyring_mock::keyring_available(log) {
                            Ok(IdentityConfiguration {
                                keyring_identity_suffix: Some(String::from(name)),
                                ..Default::default()
                            })
                        } else {
                            Ok(IdentityConfiguration {
                                encryption: Some(
                                    EncryptionConfiguration::new()
                                        .map_err(GenerateFreshEncryptionConfigurationFailed)?,
                                ),
                                ..Default::default()
                            })
                        }
                    }
                    IdentityStorageMode::PasswordProtected => Ok(IdentityConfiguration {
                        encryption: Some(
                            EncryptionConfiguration::new()
                                .map_err(GenerateFreshEncryptionConfigurationFailed)?,
                        ),
                        ..Default::default()
                    }),
                    IdentityStorageMode::Plaintext => Ok(IdentityConfiguration::default()),
                }
            }
        }

        // Use a temporary directory to prepare all identity parts in so that we don't end up with broken parts if the
        // creation process fails half-way through.
        let temp_identity_name = format!("{}{}", TEMP_IDENTITY_PREFIX, name);
        let temp_identity_dir = self.get_identity_dir_path(&temp_identity_name);
        if temp_identity_dir.exists() {
            // clean traces from previous identity creation attempts
            crate::fs::remove_dir_all(&temp_identity_dir)
                .map_err(CleanupPreviousCreationAttemptsFailed)?;
        }

        let identity_config;
        match parameters {
            IdentityCreationParameters::Pem { mode } => {
                let (pem_content, mnemonic) = generate_key()?;
                identity_config = create_identity_config(log, mode, name, None)?;
                pem_safekeeping::save_pem(
                    log,
                    self.file_locations(),
                    &temp_identity_name,
                    &identity_config,
                    pem_content.as_slice(),
                )?;
                eprintln!("Your seed phrase for identity '{name}': {}\nThis can be used to reconstruct your key in case of emergency, so write it down in a safe place.", mnemonic.phrase());
            }
            IdentityCreationParameters::PemFile { src_pem_file, mode } => {
                identity_config = create_identity_config(log, mode, name, None)?;
                let (src_pem_content, _) =
                    pem_safekeeping::load_pem_from_file(&src_pem_file, None)?;
                pem_utils::validate_pem_file(&src_pem_content)?;
                pem_safekeeping::save_pem(
                    log,
                    self.file_locations(),
                    &temp_identity_name,
                    &identity_config,
                    src_pem_content.as_slice(),
                )?;
            }
            IdentityCreationParameters::Hardware { hsm } => {
                identity_config =
                    create_identity_config(log, IdentityStorageMode::default(), name, Some(hsm))?;
                crate::fs::create_dir_all(&temp_identity_dir)
                    .map_err(CreateTemporaryIdentityDirectoryFailed)?;
            }
            IdentityCreationParameters::SeedPhrase { mnemonic, mode } => {
                identity_config = create_identity_config(log, mode, name, None)?;
                let mnemonic = Mnemonic::from_phrase(&mnemonic, Language::English)
                    .map_err(|e| CreateMnemonicFromPhraseFailed(format!("{}", e)))?;
                let key = mnemonic_to_key(&mnemonic)?;
                let pem = key
                    .to_sec1_pem(k256::pkcs8::LineEnding::CRLF)
                    .map_err(|e| ConvertSecretKeyToSec1PemFailed(Box::new(e)))?;
                let pem_content = pem.as_bytes();
                pem_safekeeping::save_pem(
                    log,
                    self.file_locations(),
                    &temp_identity_name,
                    &identity_config,
                    pem_content,
                )?;
            }
        }
        let identity_config_location = self.get_identity_json_path(&temp_identity_name);
        save_identity_configuration(log, &identity_config_location, &identity_config)?;

        // Everything is created. Now move from the temporary directory to the actual identity location.
        let identity_dir = self.get_identity_dir_path(name);
        crate::fs::rename(&temp_identity_dir, &identity_dir)
            .map_err(RenameTemporaryIdentityDirectoryFailed)?;

        if temporarily_use_anonymous_identity {
            self.use_identity_named(log, &identity_in_use)
                .map_err(|e| SwitchBackToIdentityFailed(Box::new(e)))?;
        }
        Ok(())
    }

    /// Return a sorted list of all available identity names
    pub fn get_identity_names(&self, log: &Logger) -> Result<Vec<String>, FsError> {
        let mut names = crate::fs::read_dir(self.file_locations.root())?
            .filter_map(|entry_result| match entry_result {
                Ok(dir_entry) => match dir_entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => Some(dir_entry),
                    _ => None,
                },
                _ => None,
            })
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .filter(|identity_name| self.require_identity_exists(log, identity_name).is_ok())
            .collect::<Vec<_>>();
        names.push(ANONYMOUS_IDENTITY_NAME.to_string());

        names.sort();

        Ok(names)
    }

    /// Return the name of the currently selected (active) identity
    pub fn get_selected_identity_name(&self) -> &String {
        &self.selected_identity
    }

    pub(crate) fn file_locations(&self) -> &IdentityFileLocations {
        &self.file_locations
    }

    /// Returns the pem file content of the selected identity
    pub fn export(&self, log: &Logger, name: &str) -> Result<String, ExportIdentityError> {
        self.require_identity_exists(log, name)
            .map_err(ExportIdentityError::IdentityDoesNotExist)?;
        let config = self
            .get_identity_config_or_default(name)
            .map_err(ExportIdentityError::GetIdentityConfigFailed)?;
        let (pem_content, _) = pem_safekeeping::load_pem(log, &self.file_locations, name, &config)
            .map_err(ExportIdentityError::LoadPemFailed)?;

        validate_pem_file(&pem_content).map_err(ExportIdentityError::ValidatePemFileFailed)?;
        String::from_utf8(pem_content).map_err(TranslatePemContentToTextFailed)
    }

    /// Remove a named identity.
    /// Removing the selected identity is not allowed.
    /// Removing an identity that is connected to non-ephemeral wallets is only allowed if drop_wallets is true.
    /// If display_linked_wallets_to contains a logger, this will log all the wallets the identity is connected to.
    pub fn remove(
        &self,
        log: &Logger,
        name: &str,
        drop_wallets: bool,
        display_linked_wallets_to: Option<&Logger>,
    ) -> Result<(), IdentityError> {
        self.require_identity_exists(log, name)?;

        if name == ANONYMOUS_IDENTITY_NAME {
            return Err(IdentityError::CannotDeleteAnonymousIdentity());
        }

        if self.configuration.default == name {
            return Err(IdentityError::CannotDeleteDefaultIdentity());
        }

        let wallet_config_file = self.get_persistent_wallet_config_file(name);
        if wallet_config_file.exists() {
            if let Some(logger) = display_linked_wallets_to {
                DfxIdentity::display_linked_wallets(logger, &wallet_config_file)
                    .map_err(DisplayLinkedWalletsFailed)?;
            }
            if drop_wallets {
                remove_identity_file(&wallet_config_file)?;
            } else {
                return Err(DropWalletsFlagRequiredToRemoveIdentityWithWallets());
            }
        }

        if let Ok(config) = self.get_identity_config_or_default(name) {
            if let Some(suffix) = config.keyring_identity_suffix {
                keyring_mock::delete_pem_from_keyring(&suffix)
                    .map_err(RemoveIdentityFromKeyringFailed)?;
            }
        }
        remove_identity_file(&self.get_identity_json_path(name))?;
        remove_identity_file(&self.file_locations.get_plaintext_identity_pem_path(name))?;
        remove_identity_file(&self.file_locations.get_encrypted_identity_pem_path(name))?;

        let dir = self.get_identity_dir_path(name);
        if dir.exists() {
            crate::fs::remove_dir(&dir).map_err(RemoveIdentityDirectoryFailed)?;
        }

        Ok(())
    }

    /// Rename an identity.
    /// If renaming the selected (default) identity, changes that
    /// to refer to the new identity name.
    pub fn rename(
        &mut self,
        log: &Logger,
        project_temp_dir: Option<PathBuf>,
        from: &str,
        to: &str,
    ) -> Result<bool, RenameIdentityError> {
        if to == ANONYMOUS_IDENTITY_NAME {
            return Err(RenameIdentityError::CannotCreateAnonymousIdentity());
        }
        self.require_identity_exists(log, from)
            .map_err(RenameIdentityError::IdentityDoesNotExist)?;

        let identity_config = self
            .get_identity_config_or_default(from)
            .map_err(GetIdentityConfigFailed)?;
        let from_dir = self.get_identity_dir_path(from);
        let to_dir = self.get_identity_dir_path(to);

        if to_dir.exists() {
            return Err(RenameIdentityError::IdentityAlreadyExists());
        }

        DfxIdentity::map_wallets_to_renamed_identity(project_temp_dir, from, to)
            .map_err(MapWalletsToRenamedIdentityFailed)?;
        crate::fs::rename(&from_dir, &to_dir).map_err(RenameIdentityDirectoryFailed)?;
        if let Some(keyring_identity_suffix) = &identity_config.keyring_identity_suffix {
            debug!(log, "Migrating keyring content.");
            let (pem, _) =
                pem_safekeeping::load_pem(log, &self.file_locations, from, &identity_config)
                    .map_err(LoadPemFailed)?;
            let new_config = IdentityConfiguration {
                keyring_identity_suffix: Some(to.to_string()),
                ..identity_config
            };
            pem_safekeeping::save_pem(log, &self.file_locations, to, &new_config, pem.as_ref())
                .map_err(SavePemFailed)?;
            let config_path = self.get_identity_json_path(to);
            save_identity_configuration(log, &config_path, &new_config)
                .map_err(RenameIdentityError::SaveIdentityConfigurationFailed)?;
            keyring_mock::delete_pem_from_keyring(keyring_identity_suffix)
                .map_err(RenameIdentityError::RemoveIdentityFromKeyringFailed)?;
        }

        if from == self.configuration.default {
            self.write_default_identity(to)
                .map_err(SwitchDefaultIdentitySettingsFailed)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Select an identity by name to use by default
    pub fn use_identity_named(&mut self, log: &Logger, name: &str) -> Result<(), IdentityError> {
        self.require_identity_exists(log, name)?;
        self.write_default_identity(name)?;
        self.configuration.default = name.to_string();
        Ok(())
    }

    fn write_default_identity(&self, name: &str) -> Result<(), IdentityError> {
        let config = Configuration {
            default: String::from(name),
        };
        save_configuration(&self.identity_json_path, &config)
            .map_err(IdentityError::SaveIdentityManagerConfigurationFailed)?;
        Ok(())
    }

    /// Determines if there are enough files present to consider the identity as existing.
    /// Does NOT guarantee that the identity will load correctly.
    pub fn require_identity_exists(&self, log: &Logger, name: &str) -> Result<(), IdentityError> {
        trace!(log, "Checking if identity '{name}' exists.");
        if name == ANONYMOUS_IDENTITY_NAME {
            return Ok(());
        }

        if name.starts_with(TEMP_IDENTITY_PREFIX) {
            return Err(IdentityError::ReservedIdentityName(String::from(name)));
        }

        let json_path = self.get_identity_json_path(name);
        let plaintext_pem_path = self.file_locations.get_plaintext_identity_pem_path(name);
        let encrypted_pem_path = self.file_locations.get_encrypted_identity_pem_path(name);

        if !plaintext_pem_path.exists() && !encrypted_pem_path.exists() && !json_path.exists() {
            Err(IdentityError::IdentityDoesNotExist(
                String::from(name),
                json_path,
            ))
        } else {
            Ok(())
        }
    }

    pub fn get_identity_dir_path(&self, identity: &str) -> PathBuf {
        self.file_locations.get_identity_dir_path(identity)
    }

    /// Returns the path where wallets on persistent/non-ephemeral networks are stored.
    fn get_persistent_wallet_config_file(&self, identity: &str) -> PathBuf {
        self.get_identity_dir_path(identity)
            .join(WALLET_CONFIG_FILENAME)
    }

    /// Returns the path where an identity's `IdentityConfiguration` is stored.
    pub fn get_identity_json_path(&self, identity: &str) -> PathBuf {
        self.get_identity_dir_path(identity).join(IDENTITY_JSON)
    }

    pub fn get_identity_config_or_default(
        &self,
        identity: &str,
    ) -> Result<IdentityConfiguration, IdentityError> {
        let json_path = self.get_identity_json_path(identity);
        if json_path.exists() {
            load_json_file(&json_path)
                .map_err(|err| LoadIdentityConfigurationFailed(identity.to_string(), err))
        } else {
            Ok(IdentityConfiguration::default())
        }
    }
}

pub(super) fn get_dfx_hsm_pin() -> Result<String, String> {
    std::env::var("DFX_HSM_PIN")
        .map_err(|_| "There is no DFX_HSM_PIN environment variable.".to_string())
}

fn initialize(
    logger: &Logger,
    identity_json_path: &Path,
    identity_root_path: &Path,
) -> Result<Configuration, InitializeIdentityManagerError> {
    slog::info!(
        logger,
        r#"Creating the "default" identity.
WARNING: The "default" identity is not stored securely. Do not use it to control a lot of cycles/ICP.
To create a more secure identity, create and use an identity that is protected by a password using the following commands:
    dfx identity new <my-secure-identity-name> # creates a password protected identity
    dfx identity use <my-secure-identity-name> # uses this identity by default
"#
    );

    let identity_dir = identity_root_path.join(DEFAULT_IDENTITY_NAME);
    let identity_pem_path = identity_dir.join(IDENTITY_PEM);
    if !identity_pem_path.exists() {
        if !identity_dir.exists() {
            crate::fs::create_dir_all(&identity_dir).map_err(CreateIdentityDirectoryFailed)?;
        }

        let maybe_creds_pem_path = get_legacy_creds_pem_path()?;
        if maybe_creds_pem_path
            .as_ref()
            .map(|p| p.exists())
            .unwrap_or_default()
        {
            let creds_pem_path =
                maybe_creds_pem_path.expect("Unreachable - Just checked for existence.");
            slog::info!(
                logger,
                "  - migrating key from {} to {}",
                creds_pem_path.display(),
                identity_pem_path.display()
            );
            crate::fs::copy(&creds_pem_path, &identity_pem_path)
                .map_err(MigrateLegacyIdentityFailed)?;
        } else {
            slog::info!(
                logger,
                "  - generating new key at {}",
                identity_pem_path.display()
            );
            let (key, mnemonic) = generate_key().map_err(GenerateKeyFailed)?;
            pem_safekeeping::write_pem_to_file(&identity_pem_path, None, key.as_slice())
                .map_err(WritePemToFileFailed)?;
            eprintln!("Your seed phrase: {}\nThis can be used to reconstruct your key in case of emergency, so write it down in a safe place.", mnemonic.phrase());
        }
    } else {
        slog::info!(
            logger,
            "  - using key already in place at {}",
            identity_pem_path.display()
        );
    }

    let config = Configuration {
        default: String::from(DEFAULT_IDENTITY_NAME),
    };
    save_configuration(identity_json_path, &config)
        .map_err(InitializeIdentityManagerError::SaveConfigurationFailed)?;
    slog::info!(logger, r#"Created the "default" identity."#);

    Ok(config)
}

fn get_legacy_creds_pem_path() -> Result<Option<PathBuf>, GetLegacyCredentialsPemPathError> {
    if cfg!(windows) {
        // No legacy path on Windows - there was no Windows support when paths were changed
        Ok(None)
    } else {
        let config_root = std::env::var_os("DFX_CONFIG_ROOT");
        let home = get_user_home().map_err(GetLegacyPemPathFailed)?;
        let root = config_root.unwrap_or(home);

        Ok(Some(
            PathBuf::from(root)
                .join(".dfinity")
                .join("identity")
                .join("creds.pem"),
        ))
    }
}

fn load_configuration(path: &Path) -> Result<Configuration, StructuredFileError> {
    load_json_file(path)
}

fn save_configuration(path: &Path, config: &Configuration) -> Result<(), StructuredFileError> {
    save_json_file(path, config)
}

pub(super) fn save_identity_configuration(
    log: &Logger,
    path: &Path,
    config: &IdentityConfiguration,
) -> Result<(), IdentityError> {
    trace!(log, "Writing identity configuration to {}", path.display());
    ensure_parent_dir_exists(path).map_err(EnsureIdentityConfigurationDirExistsFailed)?;

    save_json_file(path, &config).map_err(SaveIdentityConfigurationFailed)
}

/// Removes the file if it exists.
fn remove_identity_file(file: &Path) -> Result<(), IdentityError> {
    if file.exists() {
        crate::fs::remove_file(file).map_err(RemoveIdentityFileFailed)?;
    }
    Ok(())
}

/// Generates a new secp256k1 key.
pub(super) fn generate_key() -> Result<(Vec<u8>, Mnemonic), IdentityError> {
    let mnemonic = Mnemonic::new(MnemonicType::for_key_size(256).unwrap(), Language::English);
    let secret = mnemonic_to_key(&mnemonic)?;
    let pem = secret
        .to_sec1_pem(LineEnding::CRLF)
        .map_err(|e| IdentityError::GenerateFreshSecp256k1KeyFailed(Box::new(e)))?;
    Ok((pem.as_bytes().to_vec(), mnemonic))
}

pub fn mnemonic_to_key(mnemonic: &Mnemonic) -> Result<SecretKey, IdentityError> {
    const DEFAULT_DERIVATION_PATH: &str = "m/44'/223'/0'/0/0";
    let path = DEFAULT_DERIVATION_PATH.parse().unwrap();
    let seed = Seed::new(mnemonic, "");
    let pk = XPrv::derive_from_path(seed.as_bytes(), &path)
        .map_err(IdentityError::DeriveExtendedKeyFromPathFailed)?;
    Ok(SecretKey::from(pk.private_key()))
}
