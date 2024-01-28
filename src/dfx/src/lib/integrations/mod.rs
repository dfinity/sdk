use crate::lib::deps::deploy::try_create_canister;
use crate::lib::deps::PulledCanister;
use crate::lib::environment::create_agent;
use crate::lib::error::DfxResult;
use crate::lib::state_tree::canister_info::read_state_tree_canister_module_hash;
use crate::util::blob_from_arguments;
use anyhow::bail;
use candid::Principal;
use dfx_core::identity::Identity;
use dfx_core::{error::root_key::FetchRootKeyError, util::expiry_duration};
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::ManagementCanister;
use sha2::Digest;
use slog::{debug, info, Logger};
use std::time::Duration;

pub mod bitcoin;
pub mod status;

pub async fn create_integrations_agent(url: &str, logger: &Logger) -> DfxResult<Agent> {
    let timeout = expiry_duration();
    let identity = Box::new(Identity::anonymous());
    let agent = create_agent(logger.clone(), url, identity, timeout).unwrap();
    agent
        .fetch_root_key()
        .await
        .map_err(FetchRootKeyError::AgentError)?;
    Ok(agent)
}
#[context("Failed to install {name} integration canister {canister_id}")]
pub async fn initialize_integration_canister(
    agent: &Agent,
    logger: &Logger,
    name: &str,
    canister_id: Principal,
    wasm: &[u8],
    init_arg: &str,
) -> DfxResult {
    if already_installed(agent, &canister_id, wasm).await? {
        debug!(logger, "Canister {canister_id} already installed");
        return Ok(());
    }

    let pulled_canister = PulledCanister {
        name: Some(name.to_string()),
        ..Default::default()
    };
    try_create_canister(agent, logger, &canister_id, &pulled_canister).await?;

    let install_arg = blob_from_arguments(None, Some(init_arg), None, None, &None)?;
    install_canister(agent, logger, &canister_id, wasm, install_arg, name).await
}

#[context("Failed to install canister {canister_id}")]
async fn install_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    wasm: &[u8],
    install_args: Vec<u8>,
    name: &str,
) -> DfxResult {
    info!(logger, "Installing canister: {name}");
    ManagementCanister::create(agent)
        .install_code(canister_id, wasm)
        // always reinstall pulled canister
        .with_mode(InstallMode::Reinstall)
        .with_raw_arg(install_args)
        .call_and_wait()
        .await?;
    Ok(())
}

#[context("Failed to determine if canister {canister_id} is already installed")]
async fn already_installed(agent: &Agent, canister_id: &Principal, wasm: &[u8]) -> DfxResult<bool> {
    let installed_module_hash = read_state_tree_canister_module_hash(agent, *canister_id).await?;
    let expected_module_hash: [u8; 32] = sha2::Sha256::digest(wasm).into();
    let result = matches!(installed_module_hash, Some(hash) if hash == expected_module_hash);

    Ok(result)
}

#[context("Failed to wait until canister {canister_id} is installed")]
pub async fn wait_for_canister_installed(agent: &Agent, canister_id: &Principal) -> DfxResult {
    let mut retries = 0;
    loop {
        if read_state_tree_canister_module_hash(agent, *canister_id)
            .await?
            .is_some()
        {
            break;
        }
        if retries >= 60 {
            bail!("Canister {canister_id} was never installed");
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        retries += 1;
    }
    Ok(())
}
