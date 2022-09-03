use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityError};
use crate::lib::identity::{
    pem_encryption, Identity as DfxIdentity, ANONYMOUS_IDENTITY_NAME, IDENTITY_JSON, IDENTITY_PEM,
    IDENTITY_PEM_ENCRYPTED, TEMP_IDENTITY_PREFIX,
};

use anyhow::{anyhow, bail, Context};
use bip32::XPrv;
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use candid::Principal;
use fn_error_context::context;
use k256::pkcs8::LineEnding;
use k256::SecretKey;
use ring::{rand, rand::SecureRandom};
use serde::{Deserialize, Serialize};
use slog::Logger;
use std::boxed::Box;
use std::fs;
use std::path::{Path, PathBuf};

use super::identity_utils::validate_pem_file;
use super::WALLET_CONFIG_FILENAME;

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

    /// If the identity's .pem file is encrypted this contains everything (except the password) to decrypt the file.
    pub encryption: Option<EncryptionConfiguration>,
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
    /// The file path to the opensc-pkcs11 library e.g. "/usr/local/lib/opensc-pkcs11.so"
    pub pkcs11_lib_path: String,

    /// A sequence of pairs of hex digits
    pub key_id: String,
}

pub enum IdentityCreationParameters {
    Pem {
        disable_encryption: bool,
    },
    PemFile {
        src_pem_file: PathBuf,
        disable_encryption: bool,
    },
    SeedPhrase {
        mnemonic: String,
        disable_encryption: bool,
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
            mgr.require_identity_exists(identity)?;
        }

        Ok(mgr)
    }

    pub fn get_selected_identity_principal(&self) -> Option<Principal> {
        self.selected_identity_principal
    }

    /// Create an Identity instance for use with an Agent
    #[context("Failed to instantiate selected identity.")]
    pub fn instantiate_selected_identity(&mut self) -> DfxResult<Box<DfxIdentity>> {
        let name = self.selected_identity.clone();
        self.instantiate_identity_from_name(name.as_str())
    }

    /// Provide a valid Identity name and create its Identity instance for use with an Agent
    #[context("Failed to instantiate identity with name '{}'.", identity_name)]
    pub fn instantiate_identity_from_name(
        &mut self,
        identity_name: &str,
    ) -> DfxResult<Box<DfxIdentity>> {
        let identity = match identity_name {
            ANONYMOUS_IDENTITY_NAME => Box::new(DfxIdentity::anonymous()),
            identity_name => {
                self.require_identity_exists(identity_name)?;
                Box::new(DfxIdentity::load(self, identity_name)?)
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
        name: &str,
        parameters: IdentityCreationParameters,
        force: bool,
    ) -> DfxResult {
        if name == ANONYMOUS_IDENTITY_NAME {
            return Err(DfxError::new(IdentityError::CannotCreateAnonymousIdentity()));
        }

        DfxIdentity::create(self, name, parameters, force)
    }

    /// Return a sorted list of all available identity names
    #[context("Failed to list available identities.")]
    pub fn get_identity_names(&self) -> DfxResult<Vec<String>> {
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
                        .require_identity_exists(identity_name.as_ref().unwrap())
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
    pub fn export(&self, name: &str) -> DfxResult<String> {
        self.require_identity_exists(name)?;

        let config = self.get_identity_config_or_default(name)?;
        let pem_path = self.get_identity_pem_path(name, &config);
        let pem = pem_encryption::load_pem_file(&pem_path, Some(&config))?;
        validate_pem_file(&pem)?;
        String::from_utf8(pem).map_err(|e| anyhow!("Could not translate pem file to text: {}", e))
    }

    /// Remove a named identity.
    /// Removing the selected identity is not allowed.
    /// Removing an identity that is connected to non-ephemeral wallets is only allowed if drop_wallets is true.
    /// If display_linked_wallets_to contains a logger, this will log all the wallets the identity is connected to.
    #[context("Failed to remove identity '{}'.", name)]
    pub fn remove(
        &self,
        name: &str,
        drop_wallets: bool,
        display_linked_wallets_to: Option<&Logger>,
    ) -> DfxResult {
        self.require_identity_exists(name)?;

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

        remove_identity_file(&self.load_identity_pem_path(name)?)?;
        remove_identity_file(&self.get_identity_json_path(name))?;

        let dir = self.get_identity_dir_path(name);
        std::fs::remove_dir(&dir)
            .with_context(|| format!("Cannot remove identity directory at '{}'.", dir.display()))?;

        Ok(())
    }

    /// Rename an identity.
    /// If renaming the selected (default) identity, changes that
    /// to refer to the new identity name.
    #[context("Failed to rename identity '{}' to '{}'.", from, to)]
    pub fn rename(&mut self, env: &dyn Environment, from: &str, to: &str) -> DfxResult<bool> {
        if to == ANONYMOUS_IDENTITY_NAME {
            return Err(DfxError::new(IdentityError::CannotCreateAnonymousIdentity()));
        }
        self.require_identity_exists(from)?;

        let from_dir = self.get_identity_dir_path(from);
        let to_dir = self.get_identity_dir_path(to);

        if to_dir.exists() {
            return Err(DfxError::new(IdentityError::IdentityAlreadyExists()));
        }

        DfxIdentity::map_wallets_to_renamed_identity(env, from, to)?;

        std::fs::rename(&from_dir, &to_dir).map_err(|err| {
            DfxError::new(IdentityError::CannotRenameIdentityDirectory(
                from_dir,
                to_dir,
                Box::new(DfxError::new(err)),
            ))
        })?;

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
    pub fn use_identity_named(&mut self, name: &str) -> DfxResult {
        self.require_identity_exists(name)?;
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
    pub fn require_identity_exists(&self, name: &str) -> DfxResult {
        if name == ANONYMOUS_IDENTITY_NAME {
            return Ok(());
        }

        if name.starts_with(TEMP_IDENTITY_PREFIX) {
            return Err(DfxError::new(IdentityError::ReservedIdentityName(
                String::from(name),
            )));
        }

        let json_path = self.get_identity_json_path(name);
        let identity_pem_path = self.load_identity_pem_path(name)?;

        if !identity_pem_path.exists() {
            if !json_path.exists() {
                Err(DfxError::new(IdentityError::IdentityDoesNotExist(
                    String::from(name),
                    identity_pem_path,
                )))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn get_identity_dir_path(&self, identity: &str) -> PathBuf {
        self.identity_root_path.join(&identity)
    }

    /// Reads identity.json (if present) to determine where the PEM file should be at.
    /// If not present, it returns the default path.
    #[context("Failed to load identity pem path for '{}'.", identity_name)]
    pub fn load_identity_pem_path(&self, identity_name: &str) -> DfxResult<PathBuf> {
        let config = self.get_identity_config_or_default(identity_name)?;

        Ok(self.get_identity_pem_path(identity_name, &config))
    }

    /// Determines the PEM file path based on the IdentityConfiguration.
    pub fn get_identity_pem_path(
        &self,
        identity_name: &str,
        config: &IdentityConfiguration,
    ) -> PathBuf {
        let pem_file = if config.encryption.is_some() {
            IDENTITY_PEM_ENCRYPTED
        } else {
            IDENTITY_PEM
        };
        self.get_identity_dir_path(identity_name).join(pem_file)
    }

    /// Returns the path where wallets on persistent/non-ephemeral networks are stored.
    fn get_persistent_wallet_config_file(&self, identity: &str) -> PathBuf {
        self.get_identity_dir_path(identity)
            .join(WALLET_CONFIG_FILENAME)
    }

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
    dfx identity create <my-secure-identity-name> # creates a password protected identity
    dfx identity use <my-secure-identity-name> # uses this identity by default
"#
    );

    let identity_dir = identity_root_path.join(DEFAULT_IDENTITY_NAME);
    let identity_pem_path = identity_dir.join(IDENTITY_PEM);
    if !identity_pem_path.exists() {
        if !identity_dir.exists() {
            std::fs::create_dir_all(&identity_dir).map_err(|err| {
                DfxError::new(IdentityError::CannotCreateIdentityDirectory(
                    identity_dir,
                    Box::new(DfxError::new(err)),
                ))
            })?;
        }

        let creds_pem_path = get_legacy_creds_pem_path()?;
        if creds_pem_path.exists() {
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
            pem_encryption::write_pem_file(&identity_pem_path, None, key.as_slice())?;
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
fn get_legacy_creds_pem_path() -> DfxResult<PathBuf> {
    let config_root = std::env::var("DFX_CONFIG_ROOT").ok();
    let home = std::env::var("HOME")
        .map_err(|_| DfxError::new(IdentityError::CannotFindHomeDirectory()))?;
    let root = config_root.unwrap_or(home);

    Ok(PathBuf::from(root)
        .join(".dfinity")
        .join("identity")
        .join("creds.pem"))
}

#[context("Failed to load identity manager config from {}.", path.to_string_lossy())]
fn read_configuration(path: &Path) -> DfxResult<Configuration> {
    let content = std::fs::read_to_string(&path).with_context(|| {
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
    std::fs::write(&path, content).with_context(|| {
        format!(
            "Cannot write configuration file at '{}'.",
            PathBuf::from(path).display()
        )
    })?;
    Ok(())
}

#[context("Failed to read identity configuration at {}.", path.to_string_lossy())]
pub(super) fn read_identity_configuration(path: &Path) -> DfxResult<IdentityConfiguration> {
    let content = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "Cannot read identity configuration file at '{}'.",
            PathBuf::from(path).display()
        )
    })?;
    serde_json::from_str(&content).context("Failed to deserialise identity configuration.")
}

#[context("Failed to write identity configuration.")]
pub(super) fn write_identity_configuration(
    path: &Path,
    config: &IdentityConfiguration,
) -> DfxResult {
    let content = serde_json::to_string_pretty(&config)
        .context("Failed to serialize identity configuration.")?;
    std::fs::write(&path, content).with_context(|| {
        format!(
            "Cannot write identity configuration file at '{}'.",
            PathBuf::from(path).display()
        )
    })?;
    Ok(())
}

fn remove_identity_file(file: &Path) -> DfxResult {
    if file.exists() {
        std::fs::remove_file(&file)
            .with_context(|| format!("Cannot remove identity file at '{}'.", file.display()))?;
    }
    Ok(())
}

/// Generates a new secp256k1 key.
#[context("Failed to generate a fresh secp256k1 key.")]
pub(super) fn generate_key() -> DfxResult<(Vec<u8>, Mnemonic)> {
    let mnemonic = Mnemonic::new(MnemonicType::for_key_size(256)?, Language::English);
    let secret = mnemonic_to_key(&mnemonic)?;
    let pem = secret.to_pem(LineEnding::CRLF)?;
    Ok((pem.as_bytes().to_vec(), mnemonic))
}

pub fn mnemonic_to_key(mnemonic: &Mnemonic) -> DfxResult<SecretKey> {
    const DEFAULT_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";
    let seed = Seed::new(mnemonic, "");
    let pk = XPrv::derive_from_path(seed.as_bytes(), &DEFAULT_DERIVATION_PATH.parse()?)?;
    Ok(SecretKey::from(pk.private_key()))
}
