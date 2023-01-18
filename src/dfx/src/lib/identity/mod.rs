//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::config::dfinity::NetworksConfig;
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxResult, IdentityError};
use crate::lib::identity::identity_manager::IdentityStorageMode;
use crate::lib::network::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use dfx_core::error::identity::IdentityError::{
    GenerateFreshEncryptionConfigurationFailed, InstantiateHardwareIdentityFailed,
    ReadIdentityFileFailed,
};
use dfx_core::error::wallet_config::WalletConfigError;
use dfx_core::error::wallet_config::WalletConfigError::{
    EnsureWalletConfigDirFailed, LoadWalletConfigFailed, SaveWalletConfigFailed,
};
use dfx_core::json::{load_json_file, save_json_file};

use anyhow::{bail, Context};
use bip39::{Language, Mnemonic};
use candid::Principal;
use fn_error_context::context;
use ic_agent::identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity};
use ic_agent::Signature;
use ic_identity_hsm::HardwareIdentity;
use sec1::EncodeEcPrivateKey;
use serde::{Deserialize, Serialize};
use slog::{info, trace, Logger};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod identity_file_locations;
pub mod identity_manager;
pub mod identity_utils;
pub mod keyring_mock;
pub mod pem_safekeeping;
pub mod wallet;

use crate::lib::identity::identity_file_locations::IdentityFileLocations;
pub use identity_manager::{
    HardwareIdentityConfiguration, IdentityConfiguration, IdentityCreationParameters,
    IdentityManager,
};

pub const ANONYMOUS_IDENTITY_NAME: &str = "anonymous";
pub const IDENTITY_JSON: &str = "identity.json";
pub const TEMP_IDENTITY_PREFIX: &str = "___temp___";
pub const WALLET_CONFIG_FILENAME: &str = "wallets.json";
const HSM_SLOT_INDEX: usize = 0;

#[derive(Debug, Serialize, Deserialize)]
struct WalletNetworkMap {
    #[serde(flatten)]
    pub networks: BTreeMap<String, Principal>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WalletGlobalConfig {
    pub identities: BTreeMap<String, WalletNetworkMap>,
}

pub struct Identity {
    /// The name of this Identity.
    name: String,

    /// Whether this identity is stored in unencrypted form.
    /// False for identities that are not stored at all.
    pub insecure: bool,

    /// Inner implementation of this identity.
    inner: Box<dyn ic_agent::Identity + Sync + Send>,
}

impl Identity {
    /// Creates a new identity.
    ///
    /// `force`: If the identity already exists, remove it and re-create.
    pub fn create(
        log: &Logger,
        manager: &mut IdentityManager,
        name: &str,
        parameters: IdentityCreationParameters,
        force: bool,
    ) -> DfxResult {
        trace!(log, "Creating identity '{name}'.");
        let identity_in_use = manager.get_selected_identity_name().clone();
        // cannot delete an identity in use. Use anonymous identity temporarily if we force-overwrite the identity currently in use
        let temporarily_use_anonymous_identity = identity_in_use == name && force;

        if manager.require_identity_exists(log, name).is_ok() {
            trace!(log, "Identity already exists.");
            if force {
                if temporarily_use_anonymous_identity {
                    manager
                        .use_identity_named(log, ANONYMOUS_IDENTITY_NAME)
                        .context("Failed to temporarily switch over to anonymous identity.")?;
                }
                manager.remove(log, name, true, None)?;
            } else {
                bail!("Identity already exists.");
            }
        }

        fn create(identity_dir: &Path) -> DfxResult {
            std::fs::create_dir_all(identity_dir).with_context(|| {
                format!(
                    "Cannot create temporary identity directory at '{0}'.",
                    identity_dir.display(),
                )
            })
        }
        fn create_identity_config(
            log: &Logger,
            mode: IdentityStorageMode,
            name: &str,
            hardware_config: Option<HardwareIdentityConfiguration>,
        ) -> DfxResult<IdentityConfiguration> {
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
                                    identity_manager::EncryptionConfiguration::new()
                                        .map_err(GenerateFreshEncryptionConfigurationFailed)?,
                                ),
                                ..Default::default()
                            })
                        }
                    }
                    IdentityStorageMode::PasswordProtected => Ok(IdentityConfiguration {
                        encryption: Some(
                            identity_manager::EncryptionConfiguration::new()
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
        let temp_identity_dir = manager.get_identity_dir_path(&temp_identity_name);
        if temp_identity_dir.exists() {
            // clean traces from previous identity creation attempts
            std::fs::remove_dir_all(&temp_identity_dir).with_context(|| {
                format!(
                    "Failed to clean up previous creation attempts at {}.",
                    temp_identity_dir.to_string_lossy()
                )
            })?;
        }

        let identity_config;
        match parameters {
            IdentityCreationParameters::Pem { mode } => {
                let (pem_content, mnemonic) = identity_manager::generate_key()?;
                identity_config = create_identity_config(log, mode, name, None)?;
                pem_safekeeping::save_pem(
                    log,
                    manager.file_locations(),
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
                identity_utils::validate_pem_file(&src_pem_content)?;
                pem_safekeeping::save_pem(
                    log,
                    manager.file_locations(),
                    &temp_identity_name,
                    &identity_config,
                    src_pem_content.as_slice(),
                )?;
            }
            IdentityCreationParameters::Hardware { hsm } => {
                identity_config =
                    create_identity_config(log, IdentityStorageMode::default(), name, Some(hsm))?;
                create(&temp_identity_dir)?;
            }
            IdentityCreationParameters::SeedPhrase { mnemonic, mode } => {
                identity_config = create_identity_config(log, mode, name, None)?;
                let mnemonic = Mnemonic::from_phrase(&mnemonic, Language::English)?;
                let key = identity_manager::mnemonic_to_key(&mnemonic)?;
                let pem = key.to_sec1_pem(k256::pkcs8::LineEnding::CRLF)?;
                let pem_content = pem.as_bytes();
                pem_safekeeping::save_pem(
                    log,
                    manager.file_locations(),
                    &temp_identity_name,
                    &identity_config,
                    pem_content,
                )?;
            }
        }
        let identity_config_location = manager.get_identity_json_path(&temp_identity_name);
        identity_manager::write_identity_configuration(
            log,
            &identity_config_location,
            &identity_config,
        )?;

        // Everything is created. Now move from the temporary directory to the actual identity location.
        let identity_dir = manager.get_identity_dir_path(name);
        std::fs::rename(&temp_identity_dir, &identity_dir).with_context(|| {
            format!(
                "Failed to move temporary directory {} to permanent identity directory {}.",
                temp_identity_dir.to_string_lossy(),
                identity_dir.to_string_lossy()
            )
        })?;

        if temporarily_use_anonymous_identity {
            manager.use_identity_named(log, &identity_in_use)
                .with_context(||format!("Failed to switch back over to the identity you're replacing. Please run 'dfx identity use {}' to do it manually.", name))?;
        }
        Ok(())
    }

    pub fn anonymous() -> Self {
        Self {
            name: ANONYMOUS_IDENTITY_NAME.to_string(),
            inner: Box::new(AnonymousIdentity {}),
            insecure: false,
        }
    }

    fn basic(name: &str, pem_content: &[u8], was_encrypted: bool) -> Result<Self, IdentityError> {
        let inner = Box::new(
            BasicIdentity::from_pem(pem_content)
                .map_err(|e| ReadIdentityFileFailed(name.into(), Box::new(e)))?,
        );

        Ok(Self {
            name: name.to_string(),
            inner,
            insecure: !was_encrypted,
        })
    }

    fn secp256k1(
        name: &str,
        pem_content: &[u8],
        was_encrypted: bool,
    ) -> Result<Self, IdentityError> {
        let inner = Box::new(
            Secp256k1Identity::from_pem(pem_content)
                .map_err(|e| ReadIdentityFileFailed(name.into(), Box::new(e)))?,
        );

        Ok(Self {
            name: name.to_string(),
            inner,
            insecure: !was_encrypted,
        })
    }

    fn hardware(name: &str, hsm: HardwareIdentityConfiguration) -> Result<Self, IdentityError> {
        let inner = Box::new(
            HardwareIdentity::new(
                hsm.pkcs11_lib_path,
                HSM_SLOT_INDEX,
                &hsm.key_id,
                identity_manager::get_dfx_hsm_pin,
            )
            .map_err(|e| InstantiateHardwareIdentityFailed(name.into(), Box::new(e)))?,
        );
        Ok(Self {
            name: name.to_string(),
            inner,
            insecure: false,
        })
    }

    pub(crate) fn new(
        name: &str,
        config: IdentityConfiguration,
        locations: &IdentityFileLocations,
        log: &Logger,
    ) -> Result<Self, IdentityError> {
        if let Some(hsm) = config.hsm {
            Identity::hardware(name, hsm)
        } else {
            let (pem_content, was_encrypted) =
                pem_safekeeping::load_pem(log, locations, name, &config)?;
            Identity::secp256k1(name, &pem_content, was_encrypted)
                .or_else(|e| Identity::basic(name, &pem_content, was_encrypted).map_err(|_| e))
        }
    }

    /// Get the name of this identity.
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[context("Failed to get path to wallet config file for identity '{}' on network '{}'.", name, network.name)]
    pub fn get_wallet_config_file(network: &NetworkDescriptor, name: &str) -> DfxResult<PathBuf> {
        Ok(match &network.r#type {
            NetworkTypeDescriptor::Persistent => {
                // Using the global
                get_config_dfx_dir_path()?
                    .join("identity")
                    .join(name)
                    .join(WALLET_CONFIG_FILENAME)
            }
            NetworkTypeDescriptor::Ephemeral { wallet_config_path } => wallet_config_path.clone(),
        })
    }

    #[context("Failed to get wallet config for identity '{}' on network '{}'.", name, network.name)]
    fn wallet_config(
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult<(PathBuf, WalletGlobalConfig)> {
        let wallet_path = Identity::get_wallet_config_file(network, name)?;

        // Read the config file.
        Ok((
            wallet_path.clone(),
            if wallet_path.exists() {
                Identity::load_wallet_config(&wallet_path)?
            } else {
                WalletGlobalConfig {
                    identities: BTreeMap::new(),
                }
            },
        ))
    }

    /// Logs all wallets that are configured in a WalletGlobalConfig.
    pub fn display_linked_wallets(
        logger: &Logger,
        wallet_config: &Path,
    ) -> Result<(), WalletConfigError> {
        let config = Identity::load_wallet_config(wallet_config)?;
        info!(
            logger,
            "This identity is connected to the following wallets:"
        );
        for (identity, map) in config.identities {
            for (network, wallet) in map.networks {
                info!(
                    logger,
                    "    identity '{}' on network '{}' has wallet {}", identity, network, wallet
                );
            }
        }
        Ok(())
    }

    fn load_wallet_config(path: &Path) -> Result<WalletGlobalConfig, WalletConfigError> {
        load_json_file(path).map_err(LoadWalletConfigFailed)
    }

    fn save_wallet_config(
        path: &Path,
        config: &WalletGlobalConfig,
    ) -> Result<(), WalletConfigError> {
        dfx_core::fs::parent(path)
            .and_then(|path| dfx_core::fs::create_dir_all(&path))
            .map_err(EnsureWalletConfigDirFailed)?;

        save_json_file(path, &config).map_err(SaveWalletConfigFailed)
    }

    #[context("Failed to set wallet id to {} for identity '{}' on network '{}'.", id, name, network.name)]
    pub fn set_wallet_id(network: &NetworkDescriptor, name: &str, id: Principal) -> DfxResult {
        let (wallet_path, mut config) = Identity::wallet_config(network, name)?;
        // Update the wallet map in it.
        let identities = &mut config.identities;
        let network_map = identities
            .entry(name.to_string())
            .or_insert(WalletNetworkMap {
                networks: BTreeMap::new(),
            });

        network_map.networks.insert(network.name.clone(), id);

        Identity::save_wallet_config(&wallet_path, &config)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_wallet_id(network: &NetworkDescriptor, name: &str) -> DfxResult {
        let (wallet_path, mut config) = Identity::wallet_config(network, name)?;
        // Update the wallet map in it.
        let identities = &mut config.identities;
        let network_map = identities
            .entry(name.to_string())
            .or_insert(WalletNetworkMap {
                networks: BTreeMap::new(),
            });

        network_map.networks.remove(&network.name);

        std::fs::create_dir_all(wallet_path.parent().unwrap()).with_context(|| {
            format!(
                "Failed to create {}.",
                wallet_path.parent().unwrap().to_string_lossy()
            )
        })?;
        std::fs::write(
            &wallet_path,
            &serde_json::to_string_pretty(&config)
                .context("Failed to serialize global wallet config.")?,
        )
        .with_context(|| format!("Failed to write to {}.", wallet_path.to_string_lossy()))?;
        Ok(())
    }

    #[context(
        "Failed to rename '{}' to '{}' in the global wallet config.",
        original_identity,
        renamed_identity
    )]
    fn rename_wallet_global_config_key(
        original_identity: &str,
        renamed_identity: &str,
        wallet_path: PathBuf,
    ) -> DfxResult {
        let mut config = Identity::load_wallet_config(&wallet_path)
            .context("Failed to load existing wallet config.")?;
        let identities = &mut config.identities;
        let v = identities
            .remove(original_identity)
            .unwrap_or(WalletNetworkMap {
                networks: BTreeMap::new(),
            });
        identities.insert(renamed_identity.to_string(), v);
        Identity::save_wallet_config(&wallet_path, &config)?;
        Ok(())
    }

    // used for dfx identity rename foo bar
    #[context(
        "Failed to migrate wallets from identity '{}' to '{}'.",
        original_identity,
        renamed_identity
    )]
    pub fn map_wallets_to_renamed_identity(
        env: &dyn Environment,
        original_identity: &str,
        renamed_identity: &str,
    ) -> DfxResult {
        let persistent_wallet_path = get_config_dfx_dir_path()?
            .join("identity")
            .join(original_identity)
            .join(WALLET_CONFIG_FILENAME);
        if persistent_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                persistent_wallet_path,
            )?;
        }
        let shared_local_network_wallet_path =
            NetworksConfig::get_network_data_directory("local")?.join(WALLET_CONFIG_FILENAME);
        if shared_local_network_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                shared_local_network_wallet_path,
            )?;
        }
        if let Some(temp_dir) = env.get_project_temp_dir() {
            let local_wallet_path = temp_dir.join("local").join(WALLET_CONFIG_FILENAME);
            if local_wallet_path.exists() {
                Identity::rename_wallet_global_config_key(
                    original_identity,
                    renamed_identity,
                    local_wallet_path,
                )?;
            }
        }
        Ok(())
    }

    #[context("Failed to get wallet canister id for identity '{}' on network '{}'.", name, network.name)]
    pub fn wallet_canister_id(
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult<Option<Principal>> {
        let wallet_path = Identity::get_wallet_config_file(network, name)?;
        if !wallet_path.exists() {
            return Ok(None);
        }

        let config = Identity::load_wallet_config(&wallet_path)?;

        let maybe_wallet_principal = config
            .identities
            .get(name)
            .and_then(|wallet_network| wallet_network.networks.get(&network.name).cloned());
        Ok(maybe_wallet_principal)
    }
}

impl ic_agent::Identity for Identity {
    fn sender(&self) -> Result<Principal, String> {
        self.inner.sender()
    }

    fn sign(&self, blob: &[u8]) -> Result<Signature, String> {
        self.inner.sign(blob)
    }
}

impl AsRef<Identity> for Identity {
    fn as_ref(&self) -> &Identity {
        self
    }
}
