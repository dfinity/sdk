use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::deps::{
    get_pull_canisters_in_config, get_pulled_wasm_path, load_init_json, load_pulled_json,
    validate_pulled, InitJson,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::state_tree::canister_info::read_state_tree_canister_controllers;
use crate::NetworkOpt;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::{management_canister::builders::InstallMode, ManagementCanister};
use slog::{info, Logger};

/// Deploy pulled canisters.
#[derive(Parser)]
pub struct DepsDeployOpts {
    /// Specify the canister to deploy. You can specify its name (as defined in dfx.json) or Principal.
    /// If not specified, all pulled canisters will be deployed.
    canister: Option<String>,

    #[clap(flatten)]
    network: NetworkOpt,
}

pub async fn exec(env: &dyn Environment, opts: DepsDeployOpts) -> DfxResult {
    let env = create_anonymous_agent_environment(env, opts.network.network)?;

    let logger = env.get_logger();
    let pull_canisters_in_config = get_pull_canisters_in_config(&env)?;
    if pull_canisters_in_config.is_empty() {
        info!(logger, "There is no pull dependencies defined in dfx.json");
        return Ok(());
    }

    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    let pulled_json = load_pulled_json(&project_root)?;
    validate_pulled(&pulled_json, &pull_canisters_in_config)?;

    let init_json = load_init_json(&project_root)?;

    fetch_root_key_if_needed(&env).await?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    match opts.canister {
        Some(canister) => {
            let canister_id = match pull_canisters_in_config.get(&canister) {
                Some(canister_id) => *canister_id,
                None => Principal::from_text(canister).with_context(|| {
                    "The canister is not a valid Principal nor a `type: pull` canister specified in dfx.json"
                })?,
            };
            create_and_install(agent, logger, &canister_id, &init_json).await?;
        }
        None => {
            for canister_id in pulled_json.canisters.keys() {
                create_and_install(agent, logger, canister_id, &init_json).await?;
            }
        }
    }

    Ok(())
}

#[context("Failed to create and install canister {}", canister_id)]
async fn create_and_install(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    init_json: &InitJson,
) -> DfxResult {
    let arg_raw = init_json.get_arg_raw(canister_id)?;
    try_create_canister(agent, logger, canister_id).await?;
    install_pulled_canister(agent, logger, canister_id, arg_raw).await?;
    Ok(())
}

// not use operations::canister::create_canister because we don't want to modify canister_id_store
#[context("Failed to create canister {}", canister_id)]
async fn try_create_canister(agent: &Agent, logger: &Logger, canister_id: &Principal) -> DfxResult {
    info!(logger, "Creating canister: {canister_id}");
    let mgr = ManagementCanister::create(agent);
    // ignore the error that the canister is already installed
    let _res = mgr
        .create_canister()
        .as_provisional_create_with_specified_id(*canister_id)
        .call_and_wait()
        .await;
    match read_state_tree_canister_controllers(agent, *canister_id).await? {
        Some(cs) if cs.len() == 1 && cs[0] == Principal::anonymous() => Ok(()),
        Some(_) => bail!("Canister {canister_id} has been created before and its controller is not anonymous identity. Please stop and delete it and then deploy again."),
        None => bail!("Canister {canister_id} has no controllers."),
    }
}

#[context("Failed to install canister {}", canister_id)]
async fn install_pulled_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    install_args: Vec<u8>,
) -> DfxResult {
    info!(logger, "Installing canister: {canister_id}");
    let pulled_canister_path = get_pulled_wasm_path(*canister_id)?;
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
