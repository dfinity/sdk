use crate::lib::error::{DfxError, DfxResult};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::assets::wallet_wasm;
use crate::Environment;
use anyhow::{bail, Context};
use candid::Principal;
use dfx_core::canister::build_wallet_canister;
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::error::canister::CanisterBuilderError;
use dfx_core::error::wallet_config::WalletConfigError;
use dfx_core::identity::wallet::{get_wallet_config_path, wallet_canister_id};
use dfx_core::identity::{Identity, WalletGlobalConfig, WalletNetworkMap};
use ic_agent::agent::{RejectCode, RejectResponse};
use ic_agent::AgentError;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::{ManagementCanister, WalletCanister};
use slog::info;
use std::collections::BTreeMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetOrCreateWalletCanisterError {
    #[error(
        "No wallet configured for combination of identity '{identity}' and network '{network}'"
    )]
    NoWalletConfigured { identity: String, network: String },

    #[error("Failed to create wallet")]
    CreationFailed(#[source] Box<DfxError>),

    #[error(transparent)]
    WalletConfigError(#[from] WalletConfigError),

    #[error(transparent)]
    CanisterBuilderError(#[from] CanisterBuilderError),
}

/// Gets the currently configured wallet canister. If none exists yet and `create` is true, then this creates a new wallet. WARNING: Creating a new wallet costs ICP!
///
/// While developing locally, this always creates a new wallet, even if `create` is false.
/// This can be inhibited by setting the DFX_DISABLE_AUTO_WALLET env var.
pub async fn get_or_create_wallet(
    env: &dyn Environment,
    network: &NetworkDescriptor,
    name: &str,
) -> Result<Principal, GetOrCreateWalletCanisterError> {
    match wallet_canister_id(network, name)? {
        None => {
            // If the network is not the IC, we ignore the error and create a new wallet for the identity.
            if !network.is_ic && std::env::var("DFX_DISABLE_AUTO_WALLET").is_err() {
                create_wallet(env, network, name, None)
                    .await
                    .map_err(|err| GetOrCreateWalletCanisterError::CreationFailed(Box::new(err)))
            } else {
                Err(GetOrCreateWalletCanisterError::NoWalletConfigured {
                    identity: name.into(),
                    network: network.name.to_string(),
                })
            }
        }
        Some(principal) => Ok(principal),
    }
}

pub async fn create_wallet(
    env: &dyn Environment,
    network: &NetworkDescriptor,
    name: &str,
    some_canister_id: Option<Principal>,
) -> DfxResult<Principal> {
    fetch_root_key_if_needed(env).await?;
    let agent = env.get_agent();
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
                .await
                .context("Failed create canister call.")?
                .0
        }
    };

    match mgr
        .install_code(&canister_id, wasm.as_slice())
        .with_mode(InstallMode::Install)
        .await
    {
        Err(AgentError::CertifiedReject(RejectResponse {
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
pub async fn get_or_create_wallet_canister<'env>(
    env: &'env dyn Environment,
    network: &NetworkDescriptor,
    name: &str,
) -> Result<WalletCanister<'env>, GetOrCreateWalletCanisterError> {
    // without this async block, #[context] gives a spurious error
    async {
        let wallet_canister_id = get_or_create_wallet(env, network, name).await?;
        let agent = env.get_agent();
        build_wallet_canister(wallet_canister_id, agent)
            .await
            .map_err(GetOrCreateWalletCanisterError::CanisterBuilderError)
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

    Identity::save_wallet_config(&wallet_path, &config)?;
    Ok(())
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
