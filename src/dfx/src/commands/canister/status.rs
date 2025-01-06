use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::operations::canister::skip_remote_canister;
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

    /// Send request on behalf of the specified principal.
    /// This option only works for a local PocketIC instance.
    #[arg(long)]
    impersonate: Option<Principal>,

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
    let wasm_memory_threshold = if let Some(threshold) = status.settings.wasm_memory_threshold {
        format!("{} Bytes", threshold)
    } else {
        "Not Set".to_string()
    };
    let log_visibility = match status.settings.log_visibility {
        LogVisibility::Controllers => "controllers".to_string(),
        LogVisibility::Public => "public".to_string(),
        LogVisibility::AllowedViewers(viewers) => {
            if viewers.is_empty() {
                "allowed viewers list is empty".to_string()
            } else {
                let mut viewers: Vec<_> = viewers.iter().map(Principal::to_text).collect();
                viewers.sort();
                format!("allowed viewers: {}", viewers.join(", "))
            }
        }
    };

    println!(
        "\
Canister status call result for {canister}.
Status: {status}
Controllers: {controllers}
Memory allocation: {memory_allocation}
Compute allocation: {compute_allocation}
Freezing threshold: {freezing_threshold}
Idle cycles burned per day: {idle_cycles_burned_per_day}
Memory Size: {memory_size:?}
Balance: {balance} Cycles
Reserved: {reserved} Cycles
Reserved cycles limit: {reserved_cycles_limit}
Wasm memory limit: {wasm_memory_limit}
Wasm memory threshold: {wasm_memory_threshold}
Module hash: {module_hash}
Number of queries: {queries_total}
Instructions spent in queries: {query_instructions_total}
Total query request payload size (bytes): {query_req_payload_total}
Total query response payload size (bytes): {query_resp_payload_total}
Log visibility: {log_visibility}",
        status = status.status,
        controllers = controllers.join(" "),
        memory_allocation = status.settings.memory_allocation,
        compute_allocation = status.settings.compute_allocation,
        freezing_threshold = status.settings.freezing_threshold,
        idle_cycles_burned_per_day = status.idle_cycles_burned_per_day,
        memory_size = status.memory_size,
        balance = status.cycles,
        reserved = status.reserved_cycles,
        module_hash = status
            .module_hash
            .map_or_else(|| "None".to_string(), |v| format!("0x{}", hex::encode(v))),
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
    mut call_sender: &CallSender,
) -> DfxResult {
    let call_sender_override = opts.impersonate.map(CallSender::Impersonate);
    if let Some(ref call_sender_override) = call_sender_override {
        call_sender = call_sender_override;
    };

    fetch_root_key_if_needed(env).await?;

    if let Some(canister) = opts.canister.as_deref() {
        canister_status(env, canister, call_sender).await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;

        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                if skip_remote_canister(env, canister)? {
                    continue;
                }

                canister_status(env, canister, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
