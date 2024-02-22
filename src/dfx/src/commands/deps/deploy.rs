use crate::lib::deps::deploy::try_create_canister;
use crate::lib::deps::{
    get_canister_prompt, get_pull_canister_or_principal, get_pull_canisters_in_config,
    get_pulled_wasm_path, load_init_json, load_pulled_json, validate_pulled, InitJson,
    PulledCanister,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::Context;
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::{management_canister::builders::InstallMode, ManagementCanister};
use slog::{info, Logger};

/// Deploy pulled dependencies locally.
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
    validate_pulled(&pulled_json, &pull_canisters_in_config)
        .with_context(|| "Please rerun `dfx deps pull`.")?;

    let init_json = load_init_json(&project_root)?;

    fetch_root_key_if_needed(env).await?;
    let agent = env.get_agent();

    let canister_ids = match &opts.canister {
        Some(canister) => {
            let canister_id =
                get_pull_canister_or_principal(canister, &pull_canisters_in_config, &pulled_json)?;
            vec![canister_id]
        }
        None => pulled_json.canisters.keys().copied().collect(),
    };
    for canister_id in canister_ids {
        // Safe to unwrap:
        // canister_ids are guaranteed to exist in pulled.json
        let pulled_canister = pulled_json.canisters.get(&canister_id).unwrap();
        create_and_install(agent, logger, &canister_id, &init_json, pulled_canister).await?;
    }

    Ok(())
}

#[context("Failed to create and install canister {}", canister_id)]
async fn create_and_install(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    init_json: &InitJson,
    pulled_canister: &PulledCanister,
) -> DfxResult {
    let arg_raw = init_json.get_arg_raw(canister_id)?;
    try_create_canister(agent, logger, canister_id, pulled_canister).await?;
    install_pulled_canister(agent, logger, canister_id, arg_raw, pulled_canister).await?;
    Ok(())
}

#[context("Failed to install canister {}", canister_id)]
async fn install_pulled_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    install_args: Vec<u8>,
    pulled_canister: &PulledCanister,
) -> DfxResult {
    let canister_prompt = get_canister_prompt(canister_id, pulled_canister);
    info!(logger, "Installing canister: {canister_prompt}");
    let pulled_canister_path = get_pulled_wasm_path(canister_id, pulled_canister.gzip)?;
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
