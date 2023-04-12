use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::{call::AsyncCall, interfaces::ManagementCanister};
use slog::{info, Logger};

use super::{get_pull_canisters_in_config, load_pulled_json, validate_pulled};

/// Install pulled canisters.
#[derive(Parser)]
pub struct DepsDeleteOpts {
    /// Specify the canister to delete. You can specify its name (as defined in dfx.json) or Principal.
    canister: String,
}

pub async fn exec(env: &dyn Environment, opts: DepsDeleteOpts) -> DfxResult {
    let logger = env.get_logger();
    let pulled_json = load_pulled_json(env)?;
    let pull_canisters_in_config = get_pull_canisters_in_config(env)?;
    validate_pulled(&pulled_json, &pull_canisters_in_config)?;

    fetch_root_key_if_needed(env).await?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let canister_id = match pull_canisters_in_config.get(&opts.canister) {
        Some(canister_id) => *canister_id,
        None => Principal::from_text(opts.canister).with_context(|| {
            "The canister is not a valid Principal nor a name specified in dfx.json"
        })?,
    };

    if !pulled_json.canisters.contains_key(&canister_id) {
        bail!("Canister {} is not a pulled dependency.", &canister_id);
    }
    stop_and_delete_canister(agent, logger, &canister_id).await?;

    Ok(())
}

#[context("Failed to stop and delete canster {}", canister_id)]
async fn stop_and_delete_canister(
    agent: &Agent,
    logger: &Logger,
    canister_id: &Principal,
) -> DfxResult {
    info!(logger, "Deleting canister: {canister_id}");
    let mgr = ManagementCanister::create(agent);
    mgr.stop_canister(canister_id).call_and_wait().await?;
    mgr.delete_canister(canister_id).call_and_wait().await?;
    Ok(())
}
