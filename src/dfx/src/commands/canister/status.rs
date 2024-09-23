use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::LogVisibility;

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
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    let status = canister::get_canister_status(env, canister_id, call_sender).await?;

    let mut controllers: Vec<_> = status
        .settings
        .controllers
        .iter()
        .map(Principal::to_text)
        .collect();
    controllers.sort();

    let reserved_cycles_limit = if let Some(limit) = status.settings.reserved_cycles_limit {
        format!("{} Cycles", limit)
    } else {
        "Not Set".to_string()
    };

    let wasm_memory_limit = if let Some(limit) = status.settings.wasm_memory_limit {
        format!("{} Bytes", limit)
    } else {
        "Not Set".to_string()
    };
    let log_visibility = match status.settings.log_visibility {
        LogVisibility::Controllers => "controllers".to_string(),
        LogVisibility::Public => "public".to_string(),
        LogVisibility::AllowedViewers(viewers) => {
            let mut viewers: Vec<_> = viewers.iter().map(Principal::to_text).collect();
            viewers.sort();
            format!("allowed viewers: {}", viewers.join(", "))
        }
    };

    println!("Canister status call result for {canister}.\nStatus: {status}\nControllers: {controllers}\nMemory allocation: {memory_allocation}\nCompute allocation: {compute_allocation}\nFreezing threshold: {freezing_threshold}\nMemory Size: {memory_size:?}\nBalance: {balance} Cycles\nReserved: {reserved} Cycles\nReserved cycles limit: {reserved_cycles_limit}\nWasm memory limit: {wasm_memory_limit}\nModule hash: {module_hash}\nNumber of queries: {queries_total}\nInstructions spent in queries: {query_instructions_total}\nTotal query request payload size (bytes): {query_req_payload_total}\nTotal query response payload size (bytes): {query_resp_payload_total}\nLog visibility: {log_visibility}",
        status = status.status,
        controllers = controllers.join(" "),
        memory_allocation = status.settings.memory_allocation,
        compute_allocation = status.settings.compute_allocation,
        freezing_threshold = status.settings.freezing_threshold,
        memory_size = status.memory_size,
        balance = status.cycles,
        reserved = status.reserved_cycles,
        module_hash = status.module_hash.map_or_else(|| "None".to_string(), |v| format!("0x{}", hex::encode(v))),
        queries_total = status.query_stats.num_calls_total,
        query_instructions_total = status.query_stats.num_instructions_total,
        query_req_payload_total = status.query_stats.request_payload_bytes_total,
        query_resp_payload_total = status.query_stats.response_payload_bytes_total,
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
