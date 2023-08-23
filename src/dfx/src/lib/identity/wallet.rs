use crate::lib::diagnosis::DiagnosedError;
use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::assets::wallet_wasm;
use crate::Environment;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use dfx_core::canister::build_wallet_canister;
use dfx_core::config::directories::get_config_dfx_dir_path;
use dfx_core::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use dfx_core::error::wallet_config::WalletConfigError;
use dfx_core::error::wallet_config::WalletConfigError::{
    EnsureWalletConfigDirFailed, GetWalletConfigPathFailed, SaveWalletConfigFailed,
};
use dfx_core::identity::{Identity, WalletGlobalConfig, WalletNetworkMap, WALLET_CONFIG_FILENAME};
use dfx_core::json::save_json_file;
use fn_error_context::context;
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::AgentError;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::{ManagementCanister, WalletCanister};
use slog::info;
use std::collections::BTreeMap;
use std::path::PathBuf;

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
    match wallet_canister_id(network, name)? {
        None => {
            // If the network is not the IC, we ignore the error and create a new wallet for the identity.
            if !network.is_ic && std::env::var("DFX_DISABLE_AUTO_WALLET").is_err() {
                create_wallet(env, network, name, None).await
            } else {
                Err(DiagnosedError::new(format!("This command requires a configured wallet, but the combination of identity '{}' and network '{}' has no wallet set.", name, network.name),
                                        "To use an identity with a configured wallet you can do one of the following:\n\
                    - Run the command for a network where you have a wallet configured. To do so, add '--network <network name>' to your command.\n\
                    - Switch to an identity that has a wallet configured using 'dfx identity use <identity name>'.\n\
                    - Configure a wallet for this identity/network combination: 'dfx identity set-wallet <wallet id> --network <network name>'.\n\
                    - Or, if you're using mainnet, and you haven't set up a wallet yet: 'dfx quickstart'.".to_string())).context("Wallet not configured.")
            }
        }
        Some(principal) => Ok(principal),
    }
}

pub fn get_wallet_config_path(
    network: &NetworkDescriptor,
    name: &str,
) -> Result<PathBuf, WalletConfigError> {
    Ok(match &network.r#type {
        NetworkTypeDescriptor::Persistent | NetworkTypeDescriptor::Playground { .. } => {
            // Using the global
            get_config_dfx_dir_path()
                .map_err(|e| {
                    GetWalletConfigPathFailed(
                        Box::new(name.to_string()),
                        Box::new(network.name.clone()),
                        e,
                    )
                })?
                .join("identity")
                .join(name)
                .join(WALLET_CONFIG_FILENAME)
        }
        NetworkTypeDescriptor::Ephemeral { wallet_config_path } => wallet_config_path.clone(),
    })
}

#[context("Failed to create wallet for identity '{}' on network '{}'.", name, network.name)]
pub async fn create_wallet(
    env: &dyn Environment,
    network: &NetworkDescriptor,
    name: &str,
    some_canister_id: Option<Principal>,
) -> DfxResult<Principal> {
    fetch_root_key_if_needed(env).await?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let mgr = ManagementCanister::create(agent);
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
                .with_effective_canister_id(env.get_effective_canister_id())
                .call_and_wait()
                .await
                .context("Failed create canister call.")?
                .0
        }
    };

    match mgr
        .install_code(&canister_id, wasm.as_slice())
        .with_mode(InstallMode::Install)
        .call_and_wait()
        .await
    {
        Err(AgentError::ReplicaError(RejectResponse {
            reject_code: RejectCode::CanisterError,
            reject_message,
            ..
        })) if reject_message.contains("not empty") => {
            bail!(
                r#"The wallet canister "{canister_id}" already exists for user "{name}" on "{}" network."#,
                network.name
            )
        }
        res => res.context("Failed while installing wasm.")?,
    }

    let wallet = build_wallet_canister(canister_id, agent).await?;

    wallet
        .wallet_store_wallet_wasm(wasm)
        .call_and_wait()
        .await
        .context("Failed to store wallet wasm.")?;

    set_wallet_id(network, name, canister_id)?;

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
#[allow(clippy::needless_lifetimes)]
#[context("Failed to get wallet canister caller for identity '{}' on network '{}'.", name, network.name)]
pub async fn get_or_create_wallet_canister<'env>(
    env: &'env dyn Environment,
    network: &NetworkDescriptor,
    name: &str,
) -> DfxResult<WalletCanister<'env>> {
    // without this async block, #[context] gives a spurious error
    async {
        let wallet_canister_id = get_or_create_wallet(env, network, name).await?;
        let agent = env
            .get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
        build_wallet_canister(wallet_canister_id, agent)
            .await
            .map_err(Into::into)
    }
    .await
}

pub fn set_wallet_id(
    network: &NetworkDescriptor,
    name: &str,
    id: Principal,
) -> Result<(), WalletConfigError> {
    let (wallet_path, mut config) = wallet_config(network, name)?;
    // Update the wallet map in it.
    let identities = &mut config.identities;
    let network_map = identities
        .entry(name.to_string())
        .or_insert(WalletNetworkMap {
            networks: BTreeMap::new(),
        });

    network_map.networks.insert(network.name.clone(), id);

    Identity::save_wallet_config(&wallet_path, &config)
}

#[allow(dead_code)]
pub fn remove_wallet_id(network: &NetworkDescriptor, name: &str) -> Result<(), WalletConfigError> {
    let (wallet_path, mut config) = wallet_config(network, name)?;
    // Update the wallet map in it.
    let identities = &mut config.identities;
    let network_map = identities
        .entry(name.to_string())
        .or_insert(WalletNetworkMap {
            networks: BTreeMap::new(),
        });

    network_map.networks.remove(&network.name);

    dfx_core::fs::composite::ensure_parent_dir_exists(&wallet_path)
        .map_err(EnsureWalletConfigDirFailed)?;

    save_json_file(&wallet_path, &config).map_err(SaveWalletConfigFailed)
}

pub fn wallet_canister_id(
    network: &NetworkDescriptor,
    name: &str,
) -> Result<Option<Principal>, WalletConfigError> {
    let wallet_path = get_wallet_config_path(network, name)?;
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

fn wallet_config(
    network: &NetworkDescriptor,
    name: &str,
) -> Result<(PathBuf, WalletGlobalConfig), WalletConfigError> {
    let wallet_path = get_wallet_config_path(network, name)?;

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
