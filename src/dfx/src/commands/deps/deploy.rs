use crate::lib::deps::{
    get_canister_prompt, get_pull_canister_or_principal, get_pull_canisters_in_config,
    get_pulled_wasm_path, load_init_json, load_pulled_json, validate_pulled, InitJson,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::state_tree::canister_info::read_state_tree_canister_controllers;

use anyhow::{anyhow, bail};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::{management_canister::builders::InstallMode, ManagementCanister};
use slog::{info, Logger};

/// Deploy pulled dependencies.
#[derive(Parser)]
pub struct DepsDeployOpts {
    /// Specify the canister to deploy. You can specify its name (as defined in dfx.json) or Principal.
    /// If not specified, all pulled canisters will be deployed.
    canister: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: DepsDeployOpts) -> DfxResult {
    let logger = env.get_logger();
    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    if pull_canisters_in_config.is_empty() {
        info!(logger, "There are no pull dependencies defined in dfx.json");
        return Ok(());
    }

    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    let pulled_json = load_pulled_json(&project_root)?;
    validate_pulled(&pulled_json, &pull_canisters_in_config)?;

    let init_json = load_init_json(&project_root)?;

    fetch_root_key_if_needed(env).await?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let canister_ids = match &opts.canister {
        Some(canister) => {
            let canister_id = get_pull_canister_or_principal(canister, &pull_canisters_in_config)?;
            vec![canister_id]
        }
        None => pulled_json.canisters.keys().copied().collect(),
    };
    for canister_id in canister_ids {
        // Safe to unwrap:
        // caniseter_ids are guranteed to exist in pulled.json
        let pulled_canister = pulled_json.canisters.get(&canister_id).unwrap();
        let canister_prompt = get_canister_prompt(&canister_id, pulled_canister);
        create_and_install(agent, logger, &canister_id, &init_json, &canister_prompt).await?;
    }

    Ok(())
}

#[context("Failed to create and install canister {}", canister_id)]
async fn create_and_install(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    init_json: &InitJson,
    canister_prompt: &str,
) -> DfxResult {
    let arg_raw = init_json.get_arg_raw(canister_id)?;
    try_create_canister(agent, logger, canister_id, canister_prompt).await?;
    install_pulled_canister(agent, logger, canister_id, arg_raw, canister_prompt).await?;
    Ok(())
}

// not use operations::canister::create_canister because we don't want to modify canister_id_store
#[context("Failed to create canister {}", canister_id)]
async fn try_create_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    canister_prompt: &str,
) -> DfxResult {
    info!(logger, "Creating canister: {canister_prompt}");
    match read_state_tree_canister_controllers(agent, *canister_id).await? {
        Some(cs) if cs.len() == 1 && cs[0] == Principal::anonymous() => Ok(()),
        Some(_) => {
            bail!("Canister {canister_id} has been created before and its controller is not the anonymous identity. Please stop and delete it and then deploy again.");
        }
        None => {
            let mgr = ManagementCanister::create(agent);
            mgr.create_canister()
                .as_provisional_create_with_specified_id(*canister_id)
                .call_and_wait()
                .await?;
            Ok(())
        }
    }
}

#[context("Failed to install canister {}", canister_id)]
async fn install_pulled_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    install_args: Vec<u8>,
    canister_prompt: &str,
) -> DfxResult {
    info!(logger, "Installing canister: {canister_prompt}");
    let pulled_canister_path = get_pulled_wasm_path(canister_id)?;
    let wasm = dfx_core::fs::read(&pulled_canister_path)?;
    let mgr = ManagementCanister::create(agent);
    mgr.install_code(canister_id, &wasm)
        // always reinstall pulled canister
        .with_mode(InstallMode::Reinstall)
        .with_raw_arg(install_args)
        .call_and_wait()
        .await?;
    Ok(())
}
