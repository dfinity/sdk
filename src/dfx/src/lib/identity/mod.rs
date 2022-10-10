//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::config::dfinity::NetworksConfig;
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityError};
use crate::lib::identity::identity_manager::EncryptionConfiguration;
use crate::lib::network::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::{anyhow, bail, Context};
use bip39::{Language, Mnemonic};
use candid::Principal;
use fn_error_context::context;
use ic_agent::identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity};
use ic_agent::{AgentError, Signature};
use ic_identity_hsm::HardwareIdentity;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::{ManagementCanister, WalletCanister};
use serde::{Deserialize, Serialize};
use slog::{info, Logger};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

pub mod identity_manager;
pub mod identity_utils;
pub mod pem_encryption;
use crate::util::assets::wallet_wasm;
use crate::util::expiry_duration;
pub use identity_manager::{
    HardwareIdentityConfiguration, IdentityConfiguration, IdentityCreationParameters,
    IdentityManager,
};

use super::diagnosis::DiagnosedError;

pub const ANONYMOUS_IDENTITY_NAME: &str = "anonymous";
pub const IDENTITY_PEM: &str = "identity.pem";
pub const IDENTITY_PEM_ENCRYPTED: &str = "identity.pem.encrypted";
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

    /// Inner implementation of this identity.
    inner: Box<dyn ic_agent::Identity + Sync + Send>,

    /// The root directory for this identity.
    pub dir: PathBuf,
}

impl Identity {
    /// Creates a new identity.
    ///
    /// `force`: If the identity already exists, remove it and re-create.
    pub fn create(
        manager: &mut IdentityManager,
        name: &str,
        parameters: IdentityCreationParameters,
        force: bool,
    ) -> DfxResult {
        let identity_in_use = name;
        // cannot delete an identity in use. Use anonymous identity temporarily if we force-overwrite the identity currently in use
        let temporarily_use_anonymous_identity = identity_in_use == name && force;

        if manager.require_identity_exists(name).is_ok() {
            if force {
                if temporarily_use_anonymous_identity {
                    manager
                        .use_identity_named(ANONYMOUS_IDENTITY_NAME)
                        .context("Failed to temporarily switch over to anonymous identity.")?;
                }
                manager.remove(name, true, None)?;
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
        fn create_encryption_config(
            disable_encryption: bool,
        ) -> DfxResult<Option<EncryptionConfiguration>> {
            if disable_encryption {
                Ok(None)
            } else {
                Ok(Some(identity_manager::EncryptionConfiguration::new()?))
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

        let identity_config_location = manager.get_identity_json_path(&temp_identity_name);
        let mut identity_config = IdentityConfiguration::default();
        match parameters {
            IdentityCreationParameters::Pem { disable_encryption } => {
                identity_config.encryption = create_encryption_config(disable_encryption)?;
                let (pem_content, mnemonic) = identity_manager::generate_key()?;
                let pem_file = manager.get_identity_pem_path(&temp_identity_name, &identity_config);
                pem_encryption::write_pem_file(
                    &pem_file,
                    Some(&identity_config),
                    pem_content.as_slice(),
                )?;
                eprintln!("Your seed phrase for identity '{name}': {}\nThis can be used to reconstruct your key in case of emergency, so write it down in a safe place.", mnemonic.phrase());
            }
            IdentityCreationParameters::PemFile {
                src_pem_file,
                disable_encryption,
            } => {
                identity_config.encryption = create_encryption_config(disable_encryption)?;
                let src_pem_content = pem_encryption::load_pem_file(&src_pem_file, None)?;
                identity_utils::validate_pem_file(&src_pem_content)?;
                let dst_pem_file =
                    manager.get_identity_pem_path(&temp_identity_name, &identity_config);
                pem_encryption::write_pem_file(
                    &dst_pem_file,
                    Some(&identity_config),
                    src_pem_content.as_slice(),
                )?;
            }
            IdentityCreationParameters::Hardware { hsm } => {
                identity_config.hsm = Some(hsm);
                create(&temp_identity_dir)?;
            }
            IdentityCreationParameters::SeedPhrase {
                mnemonic,
                disable_encryption,
            } => {
                identity_config.encryption = create_encryption_config(disable_encryption)?;
                let mnemonic = Mnemonic::from_phrase(&mnemonic, Language::English)?;
                let key = identity_manager::mnemonic_to_key(&mnemonic)?;
                let pem_file = manager.get_identity_pem_path(&temp_identity_name, &identity_config);
                pem_encryption::write_pem_file(
                    &pem_file,
                    Some(&identity_config),
                    key.to_pem(k256::pkcs8::LineEnding::CRLF)?.as_bytes(),
                )?;
            }
        }
        identity_manager::write_identity_configuration(
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
            manager.use_identity_named(identity_in_use)
                .with_context(||format!("Failed to switch back over to the identity you're replacing. Please run 'dfx identity use {}' to do it manually.", name))?;
        }
        Ok(())
    }

    pub fn anonymous() -> Self {
        Self {
            name: ANONYMOUS_IDENTITY_NAME.to_string(),
            inner: Box::new(AnonymousIdentity {}),
            dir: PathBuf::new(),
        }
    }

    fn load_basic_identity(
        manager: &IdentityManager,
        name: &str,
        pem_content: &[u8],
    ) -> DfxResult<Self> {
        let inner = Box::new(BasicIdentity::from_pem(pem_content).map_err(|e| {
            DfxError::new(IdentityError::CannotReadIdentityFile(
                name.into(),
                Box::new(DfxError::new(e)),
            ))
        })?);

        Ok(Self {
            name: name.to_string(),
            inner,
            dir: manager.get_identity_dir_path(name),
        })
    }

    fn load_secp256k1_identity(
        manager: &IdentityManager,
        name: &str,
        pem_content: &[u8],
    ) -> DfxResult<Self> {
        let inner = Box::new(Secp256k1Identity::from_pem(pem_content).map_err(|e| {
            DfxError::new(IdentityError::CannotReadIdentityFile(
                name.into(),
                Box::new(DfxError::new(e)),
            ))
        })?);

        Ok(Self {
            name: name.to_string(),
            inner,
            dir: manager.get_identity_dir_path(name),
        })
    }

    fn load_hardware_identity(
        manager: &IdentityManager,
        name: &str,
        hsm: HardwareIdentityConfiguration,
    ) -> DfxResult<Self> {
        let inner = Box::new(
            HardwareIdentity::new(
                hsm.pkcs11_lib_path,
                HSM_SLOT_INDEX,
                &hsm.key_id,
                identity_manager::get_dfx_hsm_pin,
            )
            .map_err(DfxError::new)?,
        );
        Ok(Self {
            name: name.to_string(),
            inner,
            dir: manager.get_identity_dir_path(name),
        })
    }

    #[context("Failed to load identity '{}'.", name)]
    pub fn load(manager: &IdentityManager, name: &str) -> DfxResult<Self> {
        let json_path = manager.get_identity_json_path(name);
        let config = if json_path.exists() {
            identity_manager::read_identity_configuration(&json_path)?
        } else {
            IdentityConfiguration {
                hsm: None,
                encryption: None,
            }
        };
        if let Some(hsm) = config.hsm {
            Identity::load_hardware_identity(manager, name, hsm)
        } else {
            let pem_path = manager.load_identity_pem_path(name)?;
            let pem_content = pem_encryption::load_pem_file(&pem_path, Some(&config))?;

            Identity::load_secp256k1_identity(manager, name, &pem_content).or_else(|e| {
                Identity::load_basic_identity(manager, name, &pem_content).map_err(|_| e)
            })
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
    #[context("Failed while trying to display configured wallets in {}.", wallet_config.to_string_lossy())]
    pub fn display_linked_wallets(logger: &Logger, wallet_config: &Path) -> DfxResult {
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

    #[context("Failed to load wallet config {}.", path.to_string_lossy())]
    fn load_wallet_config(path: &Path) -> DfxResult<WalletGlobalConfig> {
        let mut buffer = Vec::new();
        std::fs::File::open(&path)
            .with_context(|| format!("Unable to open {}", path.to_string_lossy()))?
            .read_to_end(&mut buffer)
            .with_context(|| format!("Unable to read {}", path.to_string_lossy()))?;
        serde_json::from_slice::<WalletGlobalConfig>(&buffer).with_context(|| {
            format!(
                "Unable to parse contents of {} as json",
                path.to_string_lossy()
            )
        })
    }

    #[context("Failed to save wallet config to {}.", path.to_string_lossy())]
    fn save_wallet_config(path: &Path, config: &WalletGlobalConfig) -> DfxResult {
        let parent_path = match path.parent() {
            Some(parent) => parent,
            None => bail!(format!(
                "Unable to determine parent of {}",
                path.to_string_lossy()
            )),
        };
        std::fs::create_dir_all(parent_path).with_context(|| {
            format!(
                "Unable to create directory {} with parents for wallet configuration",
                parent_path.to_string_lossy()
            )
        })?;
        std::fs::write(&path, &serde_json::to_string_pretty(&config)?)
            .with_context(|| format!("Unable to write {}", path.to_string_lossy()))
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

    #[context("Failed to create wallet for identity '{}' on network '{}'.", name, network.name)]
    pub async fn create_wallet(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
        some_canister_id: Option<Principal>,
    ) -> DfxResult<Principal> {
        fetch_root_key_if_needed(env).await?;
        let mgr = ManagementCanister::create(
            env.get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
        );
        info!(
            env.get_logger(),
            "Creating a wallet canister on the {} network.", network.name
        );

        let wasm = wallet_wasm(env.get_logger())?;

        let canister_id = match some_canister_id {
            Some(id) => id,
            None => {
                mgr.create_canister()
                    .as_provisional_create_with_amount(None)
                    .call_and_wait(waiter_with_timeout(expiry_duration()))
                    .await
                    .context("Failed create canister call.")?
                    .0
            }
        };

        match mgr
            .install_code(&canister_id, wasm.as_slice())
            .with_mode(InstallMode::Install)
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await
        {
            Err(AgentError::ReplicaError {
                reject_code: 5,
                reject_message,
            }) if reject_message.contains("not empty") => {
                bail!(
                    r#"The wallet canister "{canister_id}" already exists for user "{name}" on "{}" network."#,
                    network.name
                )
            }
            res => res.context("Failed while installing wasm.")?,
        }

        let wallet = Identity::build_wallet_canister(canister_id, env).await?;

        wallet
            .wallet_store_wallet_wasm(wasm)
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await
            .context("Failed to store wallet wasm.")?;

        Identity::set_wallet_id(network, name, canister_id)?;

        info!(
            env.get_logger(),
            r#"The wallet canister on the "{}" network for user "{}" is "{}""#,
            network.name,
            name,
            canister_id,
        );

        Ok(canister_id)
    }

    /// Gets the currently configured wallet canister. If none exists yet and `create` is true, then this creates a new wallet. WARNING: Creating a new wallet costs ICP!
    ///
    /// While developing locally, this always creates a new wallet, even if `create` is false.
    /// This can be inhibited by setting the DFX_DISABLE_AUTO_WALLET env var.
    #[context("Failed to get wallet for identity '{}' on network '{}'.", name, network.name)]
    pub async fn get_or_create_wallet(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult<Principal> {
        match Identity::wallet_canister_id(network, name)? {
            None => {
                // If the network is not the IC, we ignore the error and create a new wallet for the identity.
                if !network.is_ic && std::env::var("DFX_DISABLE_AUTO_WALLET").is_err() {
                    Identity::create_wallet(env, network, name, None).await
                } else {
                    Err(DiagnosedError::new(format!("This command requires a configured wallet, but the combination of identity '{}' and network '{}' has no wallet set.", name, network.name),
                    "To use an identity with a configured wallet you can do one of the following:\n\
                    - Run the command for a network where you have a wallet configured. To do so, add '--network <network name>' to your command.\n\
                    - Switch to an identity that has a wallet configured using 'dfx identity use <identity name>'.\n\
                    - Configure a wallet for this identity/network combination: 'dfx identity set-wallet <wallet id> --network <network name>'.".to_string())).context("Wallet not configured.")
                }
            }
            Some(principal) => Ok(principal),
        }
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

    #[context("Failed to construct wallet canister caller.")]
    pub async fn build_wallet_canister(
        id: Principal,
        env: &dyn Environment,
    ) -> DfxResult<WalletCanister<'_>> {
        Ok(WalletCanister::from_canister(
            ic_utils::Canister::builder()
                .with_agent(
                    env.get_agent()
                        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
                )
                .with_canister_id(id)
                .build()
                .unwrap(),
        )
        .await?)
    }

    /// Gets the currently configured wallet canister. If none exists yet and `create` is true, then this creates a new wallet. WARNING: Creating a new wallet costs ICP!
    ///
    /// While developing locally, this always creates a new wallet, even if `create` is false.
    /// This can be inhibited by setting the DFX_DISABLE_AUTO_WALLET env var.
    #[allow(clippy::needless_lifetimes)]
    #[context("Failed to get wallet canister caller for identity '{}' on network '{}'.", name, network.name)]
    pub async fn get_or_create_wallet_canister<'env>(
        env: &'env dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult<WalletCanister<'env>> {
        let wallet_canister_id = Identity::get_or_create_wallet(env, network, name).await?;
        Identity::build_wallet_canister(wallet_canister_id, env).await
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
