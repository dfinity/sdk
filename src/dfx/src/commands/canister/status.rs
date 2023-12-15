use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::{
    DefiniteCanisterSettings, QueryStats, StatusCallResult,
};
use itertools::Itertools;
use slog::info;

/// Returns the current status of a canister: Running, Stopping, or Stopped. Also carries information like balance, current settings, memory used and everything returned by 'info'.
#[derive(Parser)]
pub struct CanisterStatusOpts {
    /// Specifies the name of the canister to return information for.
    /// You must specify either a canister name or the --all flag.
    canister: Option<String>,

    /// Returns status information for all of the canisters configured in the dfx.json file.
    #[arg(long, required_unless_present("canister"))]
    all: bool,
}

#[context("Failed to get canister status for '{}'.", canister)]
async fn canister_status(
    env: &dyn Environment,
    canister: &str,
    call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    let status = canister::get_canister_status(env, canister_id, call_sender).await?;
    let StatusCallResult {
        status,
        cycles,
        memory_size,
        module_hash,
        reserved_cycles,
        settings:
            DefiniteCanisterSettings {
                compute_allocation,
                controllers,
                freezing_threshold,
                memory_allocation,
                reserved_cycles_limit,
            },
        query_stats:
            QueryStats {
                num_calls_total,
                num_instructions_total,
                request_payload_bytes_total,
                response_payload_bytes_total,
            },
        ..
    } = status;
    let controllers = controllers
        .iter()
        .map(Principal::to_text)
        .sorted()
        .join(" ");
    let module_hash =
        module_hash.map_or_else(|| "None".to_string(), |v| format!("0x{}", hex::encode(v)));

    let reserved_cycles_limit = if let Some(limit) = reserved_cycles_limit {
        format!("{} Cycles", limit)
    } else {
        "Not Set".to_string()
    };

    info!(
        log,
        "\
Canister status call result for {canister}.
Status: {status}
Controllers: {controllers}
Memory allocation: {memory_allocation}
Compute allocation: {compute_allocation}
Freezing threshold: {freezing_threshold}
Memory Size: {memory_size}
Balance: {cycles} Cycles
Reserved: {reserved_cycles} Cycles
Reserved Cycles Limit: {reserved_cycles_limit}
Module hash: {module_hash}
- Stats -
Total query calls: {num_calls_total}
Total query call instructions: {num_instructions_total}
Total bytes in query call request payloads: {request_payload_bytes_total}
Total bytes in query call response payloads: {response_payload_bytes_total}
"
    );
    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterStatusOpts,
    call_sender: &CallSender,
) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    if let Some(canister) = opts.canister.as_deref() {
        canister_status(env, canister, call_sender).await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                canister_status(env, canister, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
