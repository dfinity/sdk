//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::config::dfinity::NetworkType;
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityError};
use crate::lib::identity::identity_manager::EncryptionConfiguration;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::{anyhow, bail, Context};
use ic_agent::identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity};
use ic_agent::Signature;
use ic_identity_hsm::HardwareIdentity;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::{ManagementCanister, Wallet};
use ic_utils::Canister;
use serde::{Deserialize, Serialize};
use slog::info;
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

pub const ANONYMOUS_IDENTITY_NAME: &str = "anonymous";
pub const IDENTITY_PEM: &str = "identity.pem";
pub const IDENTITY_PEM_ENCRYPTED: &str = "identity.pem.encrypted";
pub const IDENTITY_JSON: &str = "identity.json";
pub const TEMP_IDENTITY_PREFIX: &str = "___temp___";
const WALLET_CONFIG_FILENAME: &str = "wallets.json";
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
    pub fn create(
        manager: &IdentityManager,
        name: &str,
        parameters: IdentityCreationParameters,
    ) -> DfxResult {
        if manager.require_identity_exists(name).is_ok() {
            bail!("Identity already exists.");
        }

        fn create(identity_dir: &Path) -> DfxResult {
            std::fs::create_dir_all(identity_dir).context(format!(
                "Cannot create temporary identity directory at '{0}'.",
                identity_dir.display(),
            ))
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
            std::fs::remove_dir_all(&temp_identity_dir)?;
        }

        let identity_config_location = manager.get_identity_json_path(&temp_identity_name);
        let mut identity_config = IdentityConfiguration::default();
        match parameters {
            IdentityCreationParameters::Pem { disable_encryption } => {
                identity_config.encryption = create_encryption_config(disable_encryption)?;
                let pem_content = identity_manager::generate_key()?;
                let pem_file = manager.get_identity_pem_path(&temp_identity_name, &identity_config);
                pem_encryption::write_pem_file(
                    &pem_file,
                    Some(&identity_config),
                    pem_content.as_slice(),
                )?;
            }
            IdentityCreationParameters::PemFile {
                src_pem_file,
                disable_encryption,
            } => {
                identity_config.encryption = create_encryption_config(disable_encryption)?;
                let src_pem_content = pem_encryption::load_pem_file(&src_pem_file, None)?;
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
        }
        identity_manager::write_identity_configuration(
            &identity_config_location,
            &identity_config,
        )?;

        // Everything is created. Now move from the temporary directory to the actual identity location.
        let identity_dir = manager.get_identity_dir_path(name);
        std::fs::rename(temp_identity_dir, identity_dir)?;
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

            Identity::load_secp256k1_identity(manager, name, &pem_content)
                .or_else(|_| Identity::load_basic_identity(manager, name, &pem_content))
        }
    }

    /// Get the name of this identity.
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    fn get_wallet_config_file(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult<PathBuf> {
        Ok(match network.r#type {
            NetworkType::Persistent => {
                // Using the global
                get_config_dfx_dir_path()?
                    .join("identity")
                    .join(name)
                    .join(WALLET_CONFIG_FILENAME)
            }
            NetworkType::Ephemeral => env
                .get_temp_dir()
                .join("local")
                .join(WALLET_CONFIG_FILENAME),
        })
    }

    fn wallet_config(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult<(PathBuf, WalletGlobalConfig)> {
        let wallet_path = Identity::get_wallet_config_file(env, network, name)?;

        // Read the config file.
        Ok((
            wallet_path.clone(),
            if wallet_path.exists() {
                let mut buffer = Vec::new();
                std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
                serde_json::from_slice::<WalletGlobalConfig>(&buffer)?
            } else {
                WalletGlobalConfig {
                    identities: BTreeMap::new(),
                }
            },
        ))
    }

    pub fn set_wallet_id(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
        id: Principal,
    ) -> DfxResult {
        let (wallet_path, mut config) = Identity::wallet_config(env, network, name)?;
        // Update the wallet map in it.
        let identities = &mut config.identities;
        let network_map = identities
            .entry(name.to_string())
            .or_insert(WalletNetworkMap {
                networks: BTreeMap::new(),
            });

        network_map.networks.insert(network.name.clone(), id);

        std::fs::create_dir_all(wallet_path.parent().unwrap())?;
        std::fs::write(&wallet_path, &serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_wallet_id(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult {
        let (wallet_path, mut config) = Identity::wallet_config(env, network, name)?;
        // Update the wallet map in it.
        let identities = &mut config.identities;
        let network_map = identities
            .entry(name.to_string())
            .or_insert(WalletNetworkMap {
                networks: BTreeMap::new(),
            });

        network_map.networks.remove(&network.name);

        std::fs::create_dir_all(wallet_path.parent().unwrap())?;
        std::fs::write(&wallet_path, &serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    fn rename_wallet_global_config_key(
        original_identity: &str,
        renamed_identity: &str,
        wallet_path: PathBuf,
    ) -> DfxResult {
        let mut buffer = Vec::new();
        std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
        let mut config = serde_json::from_slice::<WalletGlobalConfig>(&buffer)?;
        let identities = &mut config.identities;
        let v = identities
            .remove(original_identity)
            .unwrap_or(WalletNetworkMap {
                networks: BTreeMap::new(),
            });
        identities.insert(renamed_identity.to_string(), v);
        std::fs::create_dir_all(wallet_path.parent().unwrap())?;
        std::fs::write(&wallet_path, &serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    // used for dfx identity rename foo bar
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
        let local_wallet_path = env
            .get_temp_dir()
            .join("local")
            .join(WALLET_CONFIG_FILENAME);
        if local_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                local_wallet_path,
            )?;
        }
        Ok(())
    }

    pub async fn create_wallet(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
        some_canister_id: Option<Principal>,
    ) -> DfxResult<Principal> {
        match Identity::wallet_canister_id(env, network, name) {
            Err(_) => {
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
                            .await?
                            .0
                    }
                };

                mgr.install_code(&canister_id, wasm.as_slice())
                    .with_mode(InstallMode::Install)
                    .call_and_wait(waiter_with_timeout(expiry_duration()))
                    .await?;

                let wallet = Identity::build_wallet_canister(canister_id, env)?;

                wallet
                    .wallet_store_wallet_wasm(wasm)
                    .call_and_wait(waiter_with_timeout(expiry_duration()))
                    .await?;

                Identity::set_wallet_id(env, network, name, canister_id)?;

                info!(
                    env.get_logger(),
                    r#"The wallet canister on the "{}" network for user "{}" is "{}""#,
                    network.name,
                    name,
                    canister_id,
                );

                Ok(canister_id)
            }
            Ok(x) => {
                info!(
                    env.get_logger(),
                    r#"The wallet canister "{}" already exists for user "{}" on "{}" network."#,
                    x.to_text(),
                    name,
                    network.name
                );
                Ok(x)
            }
        }
    }

    pub async fn get_or_create_wallet(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
        create: bool,
    ) -> DfxResult<Principal> {
        // If the network is not the IC, we ignore the error and create a new wallet for the
        // identity.
        match Identity::wallet_canister_id(env, network, name) {
            Err(_) => {
                if create {
                    Identity::create_wallet(env, network, name, None).await
                } else {
                    Err(anyhow!(
                        "Could not find wallet for \"{}\" on \"{}\" network.",
                        name,
                        network.name.clone(),
                    ))
                }
            }
            x => x,
        }
    }

    pub fn wallet_canister_id(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
    ) -> DfxResult<Principal> {
        let wallet_path = Identity::get_wallet_config_file(env, network, name)?;
        if !wallet_path.exists() {
            return Err(anyhow!(
                "Could not find wallet for \"{}\" on \"{}\" network.",
                name,
                network.name.clone(),
            ));
        }

        let config = {
            let mut buffer = Vec::new();
            std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
            serde_json::from_slice::<WalletGlobalConfig>(&buffer)?
        };

        let wallet_network = config.identities.get(name).ok_or_else(|| {
            anyhow!(
                "Could not find wallet for \"{}\" on \"{}\" network.",
                name,
                network.name.clone()
            )
        })?;
        Ok(*wallet_network.networks.get(&network.name).ok_or_else(|| {
            anyhow!(
                "Could not find wallet for \"{}\" on \"{}\" network.",
                name,
                network.name.clone()
            )
        })?)
    }

    pub fn build_wallet_canister(
        id: Principal,
        env: &dyn Environment,
    ) -> DfxResult<Canister<'_, Wallet>> {
        Ok(ic_utils::Canister::builder()
            .with_agent(
                env.get_agent()
                    .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
            )
            .with_canister_id(id)
            .with_interface(ic_utils::interfaces::Wallet)
            .build()
            .unwrap())
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn get_or_create_wallet_canister<'env>(
        env: &'env dyn Environment,
        network: &NetworkDescriptor,
        name: &str,
        create: bool,
    ) -> DfxResult<Canister<'env, Wallet>> {
        let wallet_canister_id = Identity::get_or_create_wallet(env, network, name, create).await?;
        Identity::build_wallet_canister(wallet_canister_id, env)
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
