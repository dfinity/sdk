//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::config::dfinity::NetworkType;
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityError};
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::waiter::waiter_with_timeout;
use crate::util;

use anyhow::anyhow;
use ic_agent::identity::BasicIdentity;
use ic_agent::Signature;
use ic_identity_hsm::HardwareIdentity;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::{ManagementCanister, Wallet};
use ic_utils::Canister;
use serde::{Deserialize, Serialize};
use slog::info;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;

pub mod identity_manager;
use crate::util::expiry_duration;
pub use identity_manager::{
    HardwareIdentityConfiguration, IdentityConfiguration, IdentityCreationParameters,
    IdentityManager,
};

const IDENTITY_PEM: &str = "identity.pem";
const WALLET_CONFIG_FILENAME: &str = "wallets.json";
const HSM_SLOT_ID: u32 = 0;

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
        let identity_dir = manager.get_identity_dir_path(name);

        if identity_dir.exists() {
            return Err(DfxError::new(IdentityError::IdentityAlreadyExists()));
        }
        std::fs::create_dir_all(&identity_dir).map_err(|err| {
            DfxError::new(IdentityError::CannotCreateIdentityDirectory(
                identity_dir.clone(),
                Box::new(DfxError::new(err)),
            ))
        })?;

        match parameters {
            IdentityCreationParameters::Pem() => {
                let pem_file = manager.get_identity_pem_path(name);
                identity_manager::generate_key(&pem_file)
            }
            IdentityCreationParameters::Hardware(parameters) => {
                let identity_configuration = IdentityConfiguration {
                    hsm: Some(parameters),
                };
                let json_file = manager.get_identity_json_path(name);
                identity_manager::write_identity_configuration(&json_file, &identity_configuration)
            }
        }
    }

    fn load_basic_identity(manager: &IdentityManager, name: &str) -> DfxResult<Self> {
        let dir = manager.get_identity_dir_path(name);
        let pem_path = dir.join(IDENTITY_PEM);
        let inner = Box::new(BasicIdentity::from_pem_file(&pem_path).map_err(|e| {
            DfxError::new(IdentityError::CannotReadIdentityFile(
                pem_path.clone(),
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
                HSM_SLOT_ID.into(),
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
        if json_path.exists() {
            let hsm = identity_manager::read_identity_configuration(&json_path)?
                .hsm
                .ok_or_else(|| {
                    anyhow!("No HardwareIdentityConfiguration for IdentityConfiguration.")
                })?;
            Identity::load_hardware_identity(manager, name, hsm)
        } else {
            Identity::load_basic_identity(manager, name)
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
        name: String,
    ) -> DfxResult<PathBuf> {
        Ok(match network.r#type {
            NetworkType::Persistent => {
                // Using the global
                get_config_dfx_dir_path()?
                    .join("identity")
                    .join(&name)
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
        name: String,
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
        name: String,
        id: Principal,
    ) -> DfxResult {
        let (wallet_path, mut config) = Identity::wallet_config(env, network, name.clone())?;
        // Update the wallet map in it.
        let identities = &mut config.identities;
        let network_map = identities.entry(name).or_insert(WalletNetworkMap {
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
        name: String,
    ) -> DfxResult {
        let (wallet_path, mut config) = Identity::wallet_config(env, network, name.clone())?;
        // Update the wallet map in it.
        let identities = &mut config.identities;
        let network_map = identities.entry(name).or_insert(WalletNetworkMap {
            networks: BTreeMap::new(),
        });

        network_map.networks.remove(&network.name);

        std::fs::create_dir_all(wallet_path.parent().unwrap())?;
        std::fs::write(&wallet_path, &serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    fn rename_wallet_global_config_key(
        &self,
        renamed_identity: &str,
        wallet_path: PathBuf,
    ) -> DfxResult {
        let mut buffer = Vec::new();
        std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
        let mut config = serde_json::from_slice::<WalletGlobalConfig>(&buffer)?;
        let identities = &mut config.identities;
        let v = identities.remove(&self.name).unwrap_or(WalletNetworkMap {
            networks: BTreeMap::new(),
        });
        identities.insert(renamed_identity.to_string(), v);
        std::fs::create_dir_all(wallet_path.parent().unwrap())?;
        std::fs::write(&wallet_path, &serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    // used for dfx identity rename foo bar
    pub fn map_wallets_to_renamed_identity(
        &self,
        env: &dyn Environment,
        renamed_identity: &str,
    ) -> DfxResult {
        let persistent_wallet_path = get_config_dfx_dir_path()?
            .join("identity")
            .join(&self.name)
            .join(WALLET_CONFIG_FILENAME);
        if persistent_wallet_path.exists() {
            self.rename_wallet_global_config_key(renamed_identity, persistent_wallet_path)?;
        }
        let local_wallet_path = env
            .get_temp_dir()
            .join("local")
            .join(WALLET_CONFIG_FILENAME);
        if local_wallet_path.exists() {
            self.rename_wallet_global_config_key(renamed_identity, local_wallet_path)?;
        }
        Ok(())
    }

    async fn create_wallet(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: String,
    ) -> DfxResult<Principal> {
        let mgr = ManagementCanister::create(
            env.get_agent()
                .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
        );

        info!(
            env.get_logger(),
            "Creating a wallet canister on the {} network.", network.name
        );

        let mut canister_assets = util::assets::wallet_canister()?;
        let mut wasm = Vec::new();

        for file in canister_assets.entries()? {
            let mut file = file?;
            if file.header().path()?.ends_with("wallet.wasm") {
                file.read_to_end(&mut wasm)?;
            }
        }

        let (canister_id,) = mgr
            .provisional_create_canister_with_cycles(None)
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await?;

        mgr.install_code(&canister_id, wasm.as_slice())
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await?;

        Identity::set_wallet_id(env, network, name.clone(), canister_id.clone())?;

        info!(
            env.get_logger(),
            r#"The wallet canister on the "{}" network for user "{}" is "{}""#,
            network.name,
            name,
            canister_id,
        );

        Ok(canister_id)
    }

    pub async fn get_or_create_wallet(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: String,
        create: bool,
    ) -> DfxResult<Principal> {
        // If the network is not the IC, we ignore the error and create a new wallet for the
        // identity.
        match Identity::wallet_canister_id(env, network, name.clone()) {
            Err(_) => {
                if !network.is_ic && create {
                    Identity::create_wallet(env, network, name).await
                } else {
                    Err(anyhow!(
                        "Could not find wallet {} on network {}.",
                        name.clone(),
                        network.name.clone(),
                    ))
                }
            }
            x => x,
        }
    }

    fn wallet_canister_id(
        env: &dyn Environment,
        network: &NetworkDescriptor,
        name: String,
    ) -> DfxResult<Principal> {
        let wallet_path = Identity::get_wallet_config_file(env, network, name.clone())?;
        if !wallet_path.exists() {
            return Err(anyhow!(
                "Could not find wallet {} on network {}.",
                name,
                network.name.clone(),
            ));
        }

        let config = {
            let mut buffer = Vec::new();
            std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
            serde_json::from_slice::<WalletGlobalConfig>(&buffer)?
        };

        let wallet_network = config.identities.get(&name).ok_or_else(|| {
            anyhow!(
                "Could not find wallet {} on network {}.",
                name.clone(),
                network.name.clone()
            )
        })?;
        Ok(wallet_network
            .networks
            .get(&network.name)
            .ok_or_else(|| {
                anyhow!(
                    "Could not find wallet {} on network {}.",
                    name.clone(),
                    network.name.clone()
                )
            })?
            .clone())
    }

    fn build_wallet_canister(
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

    pub async fn get_wallet_canister<'env>(
        env: &'env dyn Environment,
        network: &NetworkDescriptor,
        name: String,
    ) -> DfxResult<Canister<'_, Wallet>> {
        let wallet_canister_id = Identity::wallet_canister_id(env, network, name)?;
        Identity::build_wallet_canister(wallet_canister_id, env)
    }

    pub async fn get_or_create_wallet_canister<'env>(
        env: &'env dyn Environment,
        network: &NetworkDescriptor,
        name: String,
        create: bool,
    ) -> DfxResult<Canister<'_, Wallet>> {
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
