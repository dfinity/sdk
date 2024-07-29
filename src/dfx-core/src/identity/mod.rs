//! Identity type and module.
//!
//! Wallets are a map of network-identity, but don't have their own types or manager
//! type.
use crate::config::directories::{get_shared_network_data_directory, get_user_dfx_config_dir};
use crate::config::model::network_descriptor::NetworkDescriptor;
use crate::error::wallet_config::SaveWalletConfigError;
use crate::error::{
    identity::{
        CallSenderFromWalletError,
        CallSenderFromWalletError::{
            ParsePrincipalFromIdFailedAndGetWalletCanisterIdFailed,
            ParsePrincipalFromIdFailedAndNoWallet,
        },
        LoadPemIdentityError,
        LoadPemIdentityError::ReadIdentityFileFailed,
        MapWalletsToRenamedIdentityError,
        MapWalletsToRenamedIdentityError::RenameWalletGlobalConfigKeyFailed,
        NewHardwareIdentityError,
        NewHardwareIdentityError::InstantiateHardwareIdentityFailed,
        NewIdentityError, RenameWalletGlobalConfigKeyError,
        RenameWalletGlobalConfigKeyError::RenameWalletFailed,
    },
    wallet_config::{WalletConfigError, WalletConfigError::LoadWalletConfigFailed},
};
use crate::fs::composite::ensure_parent_dir_exists;
use crate::identity::identity_file_locations::IdentityFileLocations;
use crate::identity::wallet::wallet_canister_id;
use crate::json::{load_json_file, save_json_file};
use candid::Principal;
use ic_agent::agent::EnvelopeContent;
use ic_agent::identity::{
    AnonymousIdentity, BasicIdentity, Delegation, Secp256k1Identity, SignedDelegation,
};
use ic_agent::Signature;
use ic_identity_hsm::HardwareIdentity;
pub use identity_manager::{
    HardwareIdentityConfiguration, IdentityConfiguration, IdentityCreationParameters,
    IdentityManager,
};
use serde::{Deserialize, Serialize};
use slog::{info, Logger};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod identity_file_locations;
pub mod identity_manager;
pub mod keyring_mock;
pub mod pem_safekeeping;
pub mod pem_utils;
pub mod wallet;

pub const ANONYMOUS_IDENTITY_NAME: &str = "anonymous";
pub const IDENTITY_JSON: &str = "identity.json";
pub const TEMP_IDENTITY_PREFIX: &str = "___temp___";
pub const WALLET_CONFIG_FILENAME: &str = "wallets.json";
const HSM_SLOT_INDEX: usize = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletNetworkMap {
    #[serde(flatten)]
    pub networks: BTreeMap<String, Principal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletGlobalConfig {
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

    fn basic(
        name: &str,
        pem_content: &[u8],
        was_encrypted: bool,
    ) -> Result<Self, LoadPemIdentityError> {
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
    ) -> Result<Self, LoadPemIdentityError> {
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

    fn hardware(
        name: &str,
        hsm: HardwareIdentityConfiguration,
    ) -> Result<Self, NewHardwareIdentityError> {
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
    ) -> Result<Self, NewIdentityError> {
        if let Some(hsm) = config.hsm {
            Identity::hardware(name, hsm).map_err(NewIdentityError::NewHardwareIdentityFailed)
        } else {
            let (pem_content, was_encrypted) =
                pem_safekeeping::load_pem(log, locations, name, &config)
                    .map_err(NewIdentityError::LoadPemFailed)?;
            Identity::secp256k1(name, &pem_content, was_encrypted)
                .or_else(|e| Identity::basic(name, &pem_content, was_encrypted).map_err(|_| e))
                .map_err(NewIdentityError::LoadPemIdentityFailed)
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

    pub fn load_wallet_config(path: &Path) -> Result<WalletGlobalConfig, WalletConfigError> {
        load_json_file(path).map_err(LoadWalletConfigFailed)
    }

    pub fn save_wallet_config(
        path: &Path,
        config: &WalletGlobalConfig,
    ) -> Result<(), SaveWalletConfigError> {
        ensure_parent_dir_exists(path).map_err(SaveWalletConfigError::EnsureParentDirExists)?;

        save_json_file(path, &config).map_err(SaveWalletConfigError::SaveJsonFile)?;
        Ok(())
    }

    fn rename_wallet_global_config_key(
        original_identity: &str,
        renamed_identity: &str,
        wallet_path: PathBuf,
    ) -> Result<(), RenameWalletGlobalConfigKeyError> {
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
                    .map_err(WalletConfigError::SaveWalletConfig)
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
    ) -> Result<(), MapWalletsToRenamedIdentityError> {
        let persistent_wallet_path = get_user_dfx_config_dir()
            .map_err(MapWalletsToRenamedIdentityError::GetConfigDirectoryFailed)?
            .join("identity")
            .join(original_identity)
            .join(WALLET_CONFIG_FILENAME);
        if persistent_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                persistent_wallet_path,
            )
            .map_err(RenameWalletGlobalConfigKeyFailed)?;
        }
        let shared_local_network_wallet_path = get_shared_network_data_directory("local")
            .map_err(MapWalletsToRenamedIdentityError::GetSharedNetworkDataDirectoryFailed)?
            .join(WALLET_CONFIG_FILENAME);
        if shared_local_network_wallet_path.exists() {
            Identity::rename_wallet_global_config_key(
                original_identity,
                renamed_identity,
                shared_local_network_wallet_path,
            )
            .map_err(RenameWalletGlobalConfigKeyFailed)?;
        }
        if let Some(temp_dir) = project_temp_dir {
            let local_wallet_path = temp_dir.join("local").join(WALLET_CONFIG_FILENAME);
            if local_wallet_path.exists() {
                Identity::rename_wallet_global_config_key(
                    original_identity,
                    renamed_identity,
                    local_wallet_path,
                )
                .map_err(RenameWalletGlobalConfigKeyFailed)?;
            }
        }
        Ok(())
    }
}

impl ic_agent::Identity for Identity {
    fn sender(&self) -> Result<Principal, String> {
        self.inner.sender()
    }

    fn public_key(&self) -> Option<Vec<u8>> {
        self.inner.public_key()
    }

    fn delegation_chain(&self) -> Vec<SignedDelegation> {
        self.inner.delegation_chain()
    }

    fn sign(&self, content: &EnvelopeContent) -> Result<Signature, String> {
        self.inner.sign(content)
    }

    fn sign_arbitrary(&self, content: &[u8]) -> Result<Signature, String> {
        self.inner.sign_arbitrary(content)
    }

    fn sign_delegation(&self, content: &Delegation) -> Result<Signature, String> {
        self.inner.sign_delegation(content)
    }
}

impl AsRef<Identity> for Identity {
    fn as_ref(&self) -> &Identity {
        self
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CallSender {
    SelectedId,
    Wallet(Principal),
}

// Determine whether the selected Identity or a wallet should be the sender of the call.
// If a wallet, the principal can be selected directly, or looked up from an identity name.
impl CallSender {
    pub fn from(
        wallet_principal_or_identity_name: &Option<String>,
        network: &NetworkDescriptor,
    ) -> Result<Self, CallSenderFromWalletError> {
        let sender = if let Some(s) = wallet_principal_or_identity_name {
            match Principal::from_text(s) {
                Ok(principal) => CallSender::Wallet(principal),
                Err(principal_err) => match wallet_canister_id(network, s) {
                    Ok(Some(principal)) => CallSender::Wallet(principal),
                    Ok(None) => {
                        return Err(ParsePrincipalFromIdFailedAndNoWallet(
                            s.clone(),
                            principal_err,
                        ));
                    }
                    Err(wallet_err) => {
                        return Err(ParsePrincipalFromIdFailedAndGetWalletCanisterIdFailed(
                            s.clone(),
                            principal_err,
                            wallet_err,
                        ));
                    }
                },
            }
        } else {
            CallSender::SelectedId
        };
        Ok(sender)
    }
}
