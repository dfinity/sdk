//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::error::IdentityError;
use dfx_core::config::directories::get_shared_network_data_directory;
use dfx_core::error::identity::IdentityError::{
    GetConfigDirectoryFailed, GetSharedNetworkDataDirectoryFailed,
    InstantiateHardwareIdentityFailed, ReadIdentityFileFailed, RenameWalletFailed,
};
use dfx_core::error::wallet_config::WalletConfigError;
use dfx_core::error::wallet_config::WalletConfigError::{
    EnsureWalletConfigDirFailed, LoadWalletConfigFailed, SaveWalletConfigFailed,
};
use dfx_core::json::{load_json_file, save_json_file};

use candid::Principal;
use ic_agent::identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity};
use ic_agent::Signature;
use ic_identity_hsm::HardwareIdentity;
use serde::{Deserialize, Serialize};
use slog::{info, Logger};
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

    fn rename_wallet_global_config_key(
        original_identity: &str,
        renamed_identity: &str,
        wallet_path: PathBuf,
    ) -> Result<(), IdentityError> {
        Identity::load_wallet_config(&wallet_path)
            .and_then(|mut config| {
                let identities = &mut config.identities;
                let v = identities
                    .remove(original_identity)
                    .unwrap_or(WalletNetworkMap {
                        networks: BTreeMap::new(),
                    });
                identities.insert(renamed_identity.to_string(), v);
                Identity::save_wallet_config(&wallet_path, &config)
            })
            .map_err(|err| {
                RenameWalletFailed(
                    Box::new(original_identity.to_string()),
                    Box::new(renamed_identity.to_string()),
                    err,
                )
            })
    }

    // used for dfx identity rename foo bar
    pub fn map_wallets_to_renamed_identity(
        project_temp_dir: Option<PathBuf>,
        original_identity: &str,
        renamed_identity: &str,
    ) -> Result<(), IdentityError> {
        let persistent_wallet_path = get_config_dfx_dir_path()
            .map_err(GetConfigDirectoryFailed)?
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
        let shared_local_network_wallet_path = get_shared_network_data_directory("local")
            .map_err(GetSharedNetworkDataDirectoryFailed)?
            .join(WALLET_CONFIG_FILENAME);
        if shared_local_network_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                shared_local_network_wallet_path,
            )?;
        }
        if let Some(temp_dir) = project_temp_dir {
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
