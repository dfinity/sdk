use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityError};
use crate::lib::identity::{
    pem_safekeeping, Identity as DfxIdentity, ANONYMOUS_IDENTITY_NAME, IDENTITY_JSON, IDENTITY_PEM,
    IDENTITY_PEM_ENCRYPTED, TEMP_IDENTITY_PREFIX,
};

use anyhow::{anyhow, bail, Context};
use bip32::XPrv;
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use candid::Principal;
use dfx_core::error::identity::IdentityError::{
    CreateIdentityDirectoryFailed, RenameIdentityDirectoryFailed,
};
use fn_error_context::context;
use k256::pkcs8::LineEnding;
use k256::SecretKey;
use ring::{rand, rand::SecureRandom};
use sec1::EncodeEcPrivateKey;
use serde::{Deserialize, Serialize};
use slog::{debug, trace, Logger};
use std::boxed::Box;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use super::identity_utils::validate_pem_file;
use super::{keyring_mock, WALLET_CONFIG_FILENAME};

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
    #[context("Failed to generate a fresh EncryptionConfiguration.")]
    pub fn new() -> DfxResult<Self> {
        let mut nonce: [u8; 12] = [0; 12];
        let mut salt: [u8; 32] = [0; 32];
        let sr = rand::SystemRandom::new();
        sr.fill(&mut nonce).context("Failed to generate nonce.")?;
        sr.fill(&mut salt).context("Failed to generate salt.")?;

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

impl FromStr for IdentityStorageMode {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "keyring" => Ok(IdentityStorageMode::Keyring),
            "password-protected" => Ok(IdentityStorageMode::PasswordProtected),
            "plaintext" => Ok(IdentityStorageMode::Plaintext),
            other => bail!("Unknown storage mode: {}", other),
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
    identity_root_path: PathBuf,
    configuration: Configuration,
    selected_identity: String,
    selected_identity_principal: Option<Principal>,
}

impl IdentityManager {
    #[context("Failed to load identity manager.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        let config_dfx_dir_path = get_config_dfx_dir_path()?;
        let identity_root_path = config_dfx_dir_path.join("identity");
        let identity_json_path = config_dfx_dir_path.join("identity.json");

        let configuration = if identity_json_path.exists() {
            read_configuration(&identity_json_path)?
        } else {
            initialize(env.get_logger(), &identity_json_path, &identity_root_path)?
        };

        let identity_override = env.get_identity_override();
        let selected_identity = identity_override
            .clone()
            .unwrap_or_else(|| configuration.default.clone());

        let mgr = IdentityManager {
            identity_json_path,
            identity_root_path,
            configuration,
            selected_identity,
            selected_identity_principal: None,
        };

        if let Some(identity) = identity_override {
            mgr.require_identity_exists(env.get_logger(), identity)?;
        }

        Ok(mgr)
    }

    pub fn get_selected_identity_principal(&self) -> Option<Principal> {
        self.selected_identity_principal
    }

    /// Create an Identity instance for use with an Agent
    #[context("Failed to instantiate selected identity.")]
    pub fn instantiate_selected_identity(&mut self, log: &Logger) -> DfxResult<Box<DfxIdentity>> {
        let name = self.selected_identity.clone();
        self.instantiate_identity_from_name(name.as_str(), log)
    }

    /// Provide a valid Identity name and create its Identity instance for use with an Agent
    #[context("Failed to instantiate identity with name '{}'.", identity_name)]
    pub fn instantiate_identity_from_name(
        &mut self,
        identity_name: &str,
        log: &Logger,
    ) -> DfxResult<Box<DfxIdentity>> {
        let identity = match identity_name {
            ANONYMOUS_IDENTITY_NAME => Box::new(DfxIdentity::anonymous()),
            identity_name => {
                self.require_identity_exists(log, identity_name)?;
                Box::new(DfxIdentity::load(self, identity_name, log)?)
            }
        };
        use ic_agent::identity::Identity;
        self.selected_identity_principal =
            Some(identity.sender().map_err(|err| anyhow!("{}", err))?);
        Ok(identity)
    }

    /// Create a new identity (name -> generated key)
    ///
    /// `force`: If the identity already exists, remove and re-create it.
    #[context("Failed to create new identity '{}'.", name)]
    pub fn create_new_identity(
        &mut self,
        log: &Logger,
        name: &str,
        parameters: IdentityCreationParameters,
        force: bool,
    ) -> DfxResult {
        if name == ANONYMOUS_IDENTITY_NAME {
            return Err(DfxError::new(IdentityError::CannotCreateAnonymousIdentity()));
        }

        DfxIdentity::create(log, self, name, parameters, force)
    }

    /// Return a sorted list of all available identity names
    #[context("Failed to list available identities.")]
    pub fn get_identity_names(&self, log: &Logger) -> DfxResult<Vec<String>> {
        let mut names = self
            .identity_root_path
            .read_dir()
            .with_context(|| {
                format!(
                    "Failed to read identity root directory {}.",
                    self.identity_root_path.to_string_lossy()
                )
            })?
            .filter(|entry_result| match entry_result {
                Ok(dir_entry) => match dir_entry.file_type() {
                    Ok(file_type) => file_type.is_dir(),
                    _ => false,
                },
                _ => false,
            })
            .map(|entry_result| {
                entry_result.map(|entry| entry.file_name().to_string_lossy().to_string())
            })
            .filter(|identity_name| {
                identity_name.is_ok()
                    && self
                        .require_identity_exists(log, identity_name.as_ref().unwrap())
                        .is_ok()
            })
            .collect::<Result<Vec<_>, std::io::Error>>()
            .context("Failed to collect identity names.")?;
        names.push(ANONYMOUS_IDENTITY_NAME.to_string());

        names.sort();

        Ok(names)
    }

    /// Return the name of the currently selected (active) identity
    pub fn get_selected_identity_name(&self) -> &String {
        &self.selected_identity
    }

    /// Returns the pem file content of the selected identity
    #[context("Failed to export identity '{}'.", name)]
    pub fn export(&self, log: &Logger, name: &str) -> DfxResult<String> {
        self.require_identity_exists(log, name)?;
        let config = self.get_identity_config_or_default(name)?;
        let (pem_content, _) = pem_safekeeping::load_pem(log, self, name, &config)?;

        validate_pem_file(&pem_content)?;
        String::from_utf8(pem_content)
            .map_err(|e| anyhow!("Could not translate pem file to text: {}", e))
    }

    /// Remove a named identity.
    /// Removing the selected identity is not allowed.
    /// Removing an identity that is connected to non-ephemeral wallets is only allowed if drop_wallets is true.
    /// If display_linked_wallets_to contains a logger, this will log all the wallets the identity is connected to.
    #[context("Failed to remove identity '{}'.", name)]
    pub fn remove(
        &self,
        log: &Logger,
        name: &str,
        drop_wallets: bool,
        display_linked_wallets_to: Option<&Logger>,
    ) -> DfxResult {
        self.require_identity_exists(log, name)?;

        if name == ANONYMOUS_IDENTITY_NAME {
            return Err(DfxError::new(IdentityError::CannotDeleteAnonymousIdentity()));
        }

        if self.configuration.default == name {
            return Err(DfxError::new(IdentityError::CannotDeleteDefaultIdentity()));
        }

        let wallet_config_file = self.get_persistent_wallet_config_file(name);
        if wallet_config_file.exists() {
            if let Some(logger) = display_linked_wallets_to {
                DfxIdentity::display_linked_wallets(logger, &wallet_config_file)?;
            }
            if drop_wallets {
                remove_identity_file(&wallet_config_file)?;
            } else {
                bail!("If you want to remove an identity with configured wallets, please use the --drop-wallets flag.")
            }
        }

        if let Ok(config) = self.get_identity_config_or_default(name) {
            if let Some(suffix) = config.keyring_identity_suffix {
                keyring_mock::delete_pem_from_keyring(&suffix)?;
            }
        }
        remove_identity_file(&self.get_identity_json_path(name))?;
        remove_identity_file(&self.get_plaintext_identity_pem_path(name))?;
        remove_identity_file(&self.get_encrypted_identity_pem_path(name))?;

        let dir = self.get_identity_dir_path(name);
        if dir.exists() {
            std::fs::remove_dir(&dir).with_context(|| {
                format!("Cannot remove identity directory at '{}'.", dir.display())
            })?;
        }

        Ok(())
    }

    /// Rename an identity.
    /// If renaming the selected (default) identity, changes that
    /// to refer to the new identity name.
    #[context("Failed to rename identity '{}' to '{}'.", from, to)]
    pub fn rename(
        &mut self,
        log: &Logger,
        env: &dyn Environment,
        from: &str,
        to: &str,
    ) -> DfxResult<bool> {
        if to == ANONYMOUS_IDENTITY_NAME {
            return Err(DfxError::new(IdentityError::CannotCreateAnonymousIdentity()));
        }
        self.require_identity_exists(log, from)?;

        let identity_config = self.get_identity_config_or_default(from)?;
        let from_dir = self.get_identity_dir_path(from);
        let to_dir = self.get_identity_dir_path(to);

        if to_dir.exists() {
            return Err(DfxError::new(IdentityError::IdentityAlreadyExists()));
        }

        DfxIdentity::map_wallets_to_renamed_identity(env, from, to)?;
        dfx_core::fs::rename(&from_dir, &to_dir).map_err(RenameIdentityDirectoryFailed)?;
        if let Some(keyring_identity_suffix) = &identity_config.keyring_identity_suffix {
            debug!(log, "Migrating keyring content.");
            let (pem, _) = pem_safekeeping::load_pem(log, self, from, &identity_config)?;
            let new_config = IdentityConfiguration {
                keyring_identity_suffix: Some(to.to_string()),
                ..identity_config
            };
            pem_safekeeping::save_pem(log, self, to, &new_config, pem.as_ref())?;
            let config_path = self.get_identity_json_path(to);
            write_identity_configuration(log, &config_path, &new_config)?;
            keyring_mock::delete_pem_from_keyring(keyring_identity_suffix)?;
        }

        if from == self.configuration.default {
            self.write_default_identity(to)
                .map_err(|_| anyhow!("Failed to switch over default identity settings. Please do this manually by running 'dfx identity use {}'", to))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Select an identity by name to use by default
    #[context("Failed to switch default identity to '{}'.", name)]
    pub fn use_identity_named(&mut self, log: &Logger, name: &str) -> DfxResult {
        self.require_identity_exists(log, name)?;
        self.write_default_identity(name)?;
        self.configuration.default = name.to_string();
        Ok(())
    }

    #[context("Failed to write default identity '{}'.", name)]
    fn write_default_identity(&self, name: &str) -> DfxResult {
        let config = Configuration {
            default: String::from(name),
        };
        write_configuration(&self.identity_json_path, &config)
    }

    /// Determines if there are enough files present to consider the identity as existing.
    /// Does NOT guarantee that the identity will load correctly.
    pub fn require_identity_exists(&self, log: &Logger, name: &str) -> DfxResult {
        trace!(log, "Checking if identity '{name}' exists.");
        if name == ANONYMOUS_IDENTITY_NAME {
            return Ok(());
        }

        if name.starts_with(TEMP_IDENTITY_PREFIX) {
            return Err(DfxError::new(IdentityError::ReservedIdentityName(
                String::from(name),
            )));
        }

        let json_path = self.get_identity_json_path(name);
        let plaintext_pem_path = self.get_plaintext_identity_pem_path(name);
        let encrypted_pem_path = self.get_encrypted_identity_pem_path(name);

        if !plaintext_pem_path.exists() && !encrypted_pem_path.exists() && !json_path.exists() {
            Err(DfxError::new(IdentityError::IdentityDoesNotExist(
                String::from(name),
                json_path,
            )))
        } else {
            Ok(())
        }
    }

    pub fn get_identity_dir_path(&self, identity: &str) -> PathBuf {
        self.identity_root_path.join(identity)
    }

    /// Determines the path of the (potentially encrypted) PEM file.
    pub fn get_identity_pem_path(
        &self,
        identity_name: &str,
        identity_config: &IdentityConfiguration,
    ) -> PathBuf {
        if identity_config.encryption.is_some() {
            self.get_encrypted_identity_pem_path(identity_name)
        } else {
            self.get_plaintext_identity_pem_path(identity_name)
        }
    }

    /// Determines the path of the clear-text PEM file.
    pub fn get_plaintext_identity_pem_path(&self, identity_name: &str) -> PathBuf {
        self.get_identity_dir_path(identity_name).join(IDENTITY_PEM)
    }

    /// Determines the path of the encrypted PEM file.
    pub fn get_encrypted_identity_pem_path(&self, identity_name: &str) -> PathBuf {
        self.get_identity_dir_path(identity_name)
            .join(IDENTITY_PEM_ENCRYPTED)
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

    #[context("Failed to get identity config for '{}'.", identity)]
    pub fn get_identity_config_or_default(
        &self,
        identity: &str,
    ) -> DfxResult<IdentityConfiguration> {
        let json_path = self.get_identity_json_path(identity);
        if json_path.exists() {
            let content = std::fs::read(&json_path)
                .with_context(|| format!("Failed to read {}.", json_path.to_string_lossy()))?;
            let config = serde_json::from_slice(content.as_ref()).with_context(|| {
                format!(
                    "Error deserializing identity configuration at {}.",
                    json_path.to_string_lossy()
                )
            })?;
            Ok(config)
        } else {
            Ok(IdentityConfiguration::default())
        }
    }
}

pub(super) fn get_dfx_hsm_pin() -> Result<String, String> {
    std::env::var("DFX_HSM_PIN")
        .map_err(|_| "There is no DFX_HSM_PIN environment variable.".to_string())
}

#[context("Failed to initialize identity manager at {}.", identity_root_path.to_string_lossy())]
fn initialize(
    logger: &Logger,
    identity_json_path: &Path,
    identity_root_path: &Path,
) -> DfxResult<Configuration> {
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
            dfx_core::fs::create_dir_all(&identity_dir).map_err(CreateIdentityDirectoryFailed)?;
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
            fs::copy(&creds_pem_path, &identity_pem_path).with_context(|| {
                format!(
                    "Failed to migrate legacy identity from {} to {}.",
                    creds_pem_path.to_string_lossy(),
                    identity_pem_path.to_string_lossy()
                )
            })?;
        } else {
            slog::info!(
                logger,
                "  - generating new key at {}",
                identity_pem_path.display()
            );
            let (key, mnemonic) = generate_key()?;
            pem_safekeeping::write_pem_to_file(&identity_pem_path, None, key.as_slice())?;
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
    write_configuration(identity_json_path, &config)?;
    slog::info!(logger, r#"Created the "default" identity."#);

    Ok(config)
}

#[context("Failed to get legacy pem path.")]
fn get_legacy_creds_pem_path() -> DfxResult<Option<PathBuf>> {
    if cfg!(windows) {
        // No legacy path on Windows - there was no Windows support when paths were changed
        Ok(None)
    } else {
        let config_root = std::env::var("DFX_CONFIG_ROOT").ok();
        let home = std::env::var("HOME").map_err(|_| IdentityError::NoHomeInEnvironment())?;
        let root = config_root.unwrap_or(home);

        Ok(Some(
            PathBuf::from(root)
                .join(".dfinity")
                .join("identity")
                .join("creds.pem"),
        ))
    }
}

#[context("Failed to load identity manager config from {}.", path.to_string_lossy())]
fn read_configuration(path: &Path) -> DfxResult<Configuration> {
    let content = std::fs::read_to_string(path).with_context(|| {
        format!(
            "Cannot read configuration file at '{}'.",
            PathBuf::from(path).display()
        )
    })?;
    serde_json::from_str(&content).map_err(DfxError::from)
}

#[context("Failed to write configuration to {}.", path.to_string_lossy())]
fn write_configuration(path: &Path, config: &Configuration) -> DfxResult {
    let content =
        serde_json::to_string_pretty(&config).context("Failed to serialize configuration.")?;
    std::fs::write(path, content).with_context(|| {
        format!(
            "Cannot write configuration file at '{}'.",
            PathBuf::from(path).display()
        )
    })?;
    Ok(())
}

#[context("Failed to read identity configuration at {}.", path.to_string_lossy())]
pub(super) fn read_identity_configuration(path: &Path) -> DfxResult<IdentityConfiguration> {
    let content = std::fs::read_to_string(path).with_context(|| {
        format!(
            "Cannot read identity configuration file at '{}'.",
            PathBuf::from(path).display()
        )
    })?;
    serde_json::from_str(&content).context("Failed to deserialise identity configuration.")
}

#[context("Failed to write identity configuration.")]
pub(super) fn write_identity_configuration(
    log: &Logger,
    path: &Path,
    config: &IdentityConfiguration,
) -> DfxResult {
    trace!(log, "Writing identity configuration to {}", path.display());
    let content = serde_json::to_string_pretty(&config)
        .context("Failed to serialize identity configuration.")?;
    std::fs::create_dir_all(path.parent().with_context(|| {
        format!(
            "Failed to determine parent of identity configuration file {}",
            PathBuf::from(path).display(),
        )
    })?)
    .with_context(|| {
        format!(
            "Failed to create directory for identity configuration file {}",
            PathBuf::from(path).display()
        )
    })?;
    std::fs::write(path, content).with_context(|| {
        format!(
            "Cannot write identity configuration file at '{}'.",
            PathBuf::from(path).display()
        )
    })?;
    Ok(())
}

/// Removes the file if it exists.
fn remove_identity_file(file: &Path) -> DfxResult {
    if file.exists() {
        std::fs::remove_file(file)
            .with_context(|| format!("Cannot remove identity file at '{}'.", file.display()))?;
    }
    Ok(())
}

/// Generates a new secp256k1 key.
#[context("Failed to generate a fresh secp256k1 key.")]
pub(super) fn generate_key() -> DfxResult<(Vec<u8>, Mnemonic)> {
    let mnemonic = Mnemonic::new(MnemonicType::for_key_size(256)?, Language::English);
    let secret = mnemonic_to_key(&mnemonic)?;
    let pem = secret.to_sec1_pem(LineEnding::CRLF)?;
    Ok((pem.as_bytes().to_vec(), mnemonic))
}

pub fn mnemonic_to_key(mnemonic: &Mnemonic) -> DfxResult<SecretKey> {
    const DEFAULT_DERIVATION_PATH: &str = "m/44'/223'/0'/0/0";
    let seed = Seed::new(mnemonic, "");
    let pk = XPrv::derive_from_path(seed.as_bytes(), &DEFAULT_DERIVATION_PATH.parse()?)?;
    Ok(SecretKey::from(pk.private_key()))
}
