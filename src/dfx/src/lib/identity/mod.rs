//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::config::dfinity::NetworkType;
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityErrorKind};
use crate::lib::network::network_descriptor::NetworkDescriptor;
use ic_agent::identity::BasicIdentity;
use ic_agent::Signature;
use ic_types::Principal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub mod identity_manager;
use ic_utils::interfaces::Wallet;
use ic_utils::Canister;
pub use identity_manager::IdentityManager;
use std::io::Read;

const IDENTITY_PEM: &str = "identity.pem";
const WALLET_CONFIG_FILENAME: &str = "wallets.json";

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

    /// The directory where files for this identity can be found.
    dir: PathBuf,

    /// Inner implementation of this identity.
    inner: Box<dyn ic_agent::Identity + Sync + Send>,
}

impl Identity {
    pub fn create(manager: &IdentityManager, name: &str) -> DfxResult<Self> {
        let identity_dir = manager.get_identity_dir_path(name);

        if identity_dir.exists() {
            return Err(DfxError::IdentityError(
                IdentityErrorKind::IdentityAlreadyExists(),
            ));
        }
        std::fs::create_dir_all(&identity_dir).map_err(|e| {
            DfxError::IdentityError(IdentityErrorKind::CouldNotCreateIdentityDirectory(
                identity_dir.clone(),
                e,
            ))
        })?;

        let pem_file = identity_dir.join(IDENTITY_PEM);
        identity_manager::generate_key(&pem_file)?;

        Self::load(manager, name)
    }

    pub fn load(manager: &IdentityManager, name: &str) -> DfxResult<Self> {
        let dir = manager.get_identity_dir_path(name);
        let pem_path = dir.join(IDENTITY_PEM);
        let inner = Box::new(BasicIdentity::from_pem_file(&pem_path).map_err(|e| {
            DfxError::IdentityError(IdentityErrorKind::AgentPemError(e, pem_path.clone()))
        })?);

        Ok(Self {
            name: name.to_string(),
            dir,
            inner,
        })
    }

    /// Get the name of this identity.
    pub fn name(&self) -> &str {
        &self.name
    }

    fn get_wallet_config_file(
        &self,
        env: &dyn Environment,
        network: &NetworkDescriptor,
    ) -> DfxResult<PathBuf> {
        Ok(match network.r#type {
            NetworkType::Persistent => {
                // Using the global
                get_config_dfx_dir_path()?
                    .join("wallets")
                    .join(WALLET_CONFIG_FILENAME)
            }
            NetworkType::Ephemeral => env
                .get_temp_dir()
                .join("local")
                .join(WALLET_CONFIG_FILENAME),
        })
    }

    pub fn set_wallet_id(
        &self,
        env: &dyn Environment,
        network: &NetworkDescriptor,
        id: Principal,
    ) -> DfxResult {
        let wallet_path = self.get_wallet_config_file(env, network)?;

        // Read the config file.
        let mut config = if wallet_path.exists() {
            let mut buffer = Vec::new();
            std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
            serde_json::from_slice::<WalletGlobalConfig>(&buffer)?
        } else {
            WalletGlobalConfig {
                identities: BTreeMap::new(),
            }
        };

        // Update the wallet map in it.
        let identities = &mut config.identities;
        let network_map = identities
            .entry(self.name.clone())
            .or_insert(WalletNetworkMap {
                networks: BTreeMap::new(),
            });

        network_map.networks.insert(network.name.clone(), id);

        std::fs::create_dir_all(wallet_path.parent().unwrap())?;
        std::fs::write(&wallet_path, &serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    pub fn wallet_canister_id(
        &self,
        env: &dyn Environment,
        network: &NetworkDescriptor,
    ) -> DfxResult<Principal> {
        let wallet_path = self.get_wallet_config_file(env, network)?;
        if !wallet_path.exists() {
            return Err(DfxError::CouldNotFindWalletForIdentity(
                self.name.clone(),
                network.name.clone(),
            ));
        }

        let config = {
            let mut buffer = Vec::new();
            std::fs::File::open(&wallet_path)?.read_to_end(&mut buffer)?;
            serde_json::from_slice::<WalletGlobalConfig>(&buffer)?
        };

        let wallet_network = config.identities.get(&self.name).ok_or_else(|| {
            DfxError::CouldNotFindWalletForIdentity(self.name.clone(), network.name.clone())
        })?;
        Ok(wallet_network
            .networks
            .get(&network.name)
            .ok_or_else(|| {
                DfxError::CouldNotFindWalletForIdentity(self.name.clone(), network.name.clone())
            })?
            .clone())
    }

    pub fn get_wallet<'env>(
        &self,
        env: &'env dyn Environment,
        network: &NetworkDescriptor,
    ) -> DfxResult<Canister<'env, Wallet>> {
        Ok(ic_utils::Canister::builder()
            .with_agent(
                env.get_agent()
                    .ok_or(DfxError::CommandMustBeRunInAProject)?,
            )
            .with_canister_id(self.wallet_canister_id(env, network)?)
            .with_interface(ic_utils::interfaces::Wallet)
            .build()
            .unwrap())
    }
}

impl ic_agent::Identity for Identity {
    fn sender(&self) -> Result<Principal, String> {
        self.inner.sender()
    }

    fn sign(&self, blob: &[u8], principal: &Principal) -> Result<Signature, String> {
        self.inner.sign(blob, principal)
    }
}

impl AsRef<Identity> for Identity {
    fn as_ref(&self) -> &Identity {
        self
    }
}
