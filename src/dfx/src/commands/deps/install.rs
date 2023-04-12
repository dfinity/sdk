use crate::lib::deps::InitJson;
use crate::lib::deps::{
    get_pull_canisters_in_config, get_pulled_wasm_path, load_init_json, load_pulled_json,
    validate_pulled,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::operations::canister::create_canister;
use crate::lib::root_key::fetch_root_key_if_needed;

use std::collections::BTreeSet;

use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::{management_canister::builders::InstallMode, ManagementCanister};
use slog::{info, Logger};

/// Install pulled canisters.
#[derive(Parser)]
pub struct DepsInstallOpts {
    /// Specify the canister to install. You can specify its name (as defined in dfx.json) or Principal.
    /// If not specified, all pulled canisters will be installed.
    canister: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: DepsInstallOpts) -> DfxResult {
    let logger = env.get_logger();
    let project_root = env.get_config_or_anyhow()?.get_project_root().to_path_buf();
    let pulled_json = load_pulled_json(&project_root)?;
    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    validate_pulled(&pulled_json, &pull_canisters_in_config)?;

    let init_json = load_init_json(&project_root)?;

    fetch_root_key_if_needed(env).await?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    match opts.canister {
        Some(canister) => {
            let (name, canister_id) = match pull_canisters_in_config.get(&canister) {
                Some(canister_id) => (Some(canister.as_str()), *canister_id),
                None => {
                    let canister_id = Principal::from_text(canister).with_context(|| {
                        "The canister is not a valid Principal nor a name specified in dfx.json"
                    })?;
                    (None, canister_id)
                }
            };
            create_and_install(env, agent, logger, name, &canister_id, &init_json).await?;
        }
        None => {
            let mut installed_canisters = BTreeSet::new();
            for (name, canister_id) in &pull_canisters_in_config {
                create_and_install(env, agent, logger, Some(name), canister_id, &init_json).await?;
                installed_canisters.insert(canister_id);
            }
            for canister_id in pulled_json.canisters.keys() {
                if !installed_canisters.contains(canister_id) {
                    create_and_install(env, agent, logger, None, canister_id, &init_json).await?;
                }
            }
        }
    }

    Ok(())
}

#[context("Failed to create and install canster {}", canister_id)]
async fn create_and_install(
    env: &dyn Environment,
    agent: &Agent,
    logger: &Logger,
    name: Option<&str>,
    canister_id: &Principal,
    init_json: &InitJson,
) -> DfxResult {
    let arg_raw = init_json.get_arg_raw(canister_id)?;
    try_create_canister(env, name, canister_id).await?;
    install_pulled_canister(agent, logger, canister_id, arg_raw).await?;
    Ok(())
}

#[context("Failed to create canster {}", canister_id)]
async fn try_create_canister(
    env: &dyn Environment,
    name: Option<&str>,
    canister_id: &Principal,
) -> DfxResult {
    let default_name = format!("deps:{canister_id}");
    let canister_name = match name {
        Some(s) => s,
        None => &default_name,
    };
    // Ignore the error that canister is already created before
    let _res = create_canister(
        env,
        canister_name,
        None,
        Some(*canister_id),
        &CallSender::SelectedId,
        CanisterSettings::default(),
    )
    .await;
    Ok(())
}

#[context("Failed to install canster {}", canister_id)]
async fn install_pulled_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
    install_args: Vec<u8>,
) -> DfxResult {
    info!(logger, "Installing canister: {canister_id}");
    let pulled_canister_path = get_pulled_wasm_path(*canister_id)?;
    let wasm = std::fs::read(pulled_canister_path)?;
    let mgr = ManagementCanister::create(agent);
    mgr.install_code(canister_id, &wasm)
        // always reinstall pulled canister
        .with_mode(InstallMode::Reinstall)
        .with_raw_arg(install_args)
        .call_and_wait()
        .await?;
    Ok(())
}
