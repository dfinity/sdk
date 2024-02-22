use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use slog::info;

/// Get the canister logs.
#[derive(Parser)]
pub struct TailOpts {
    /// Specifies the name or id of the canister to get its canister information.
    canister: String,
}

pub async fn exec(env: &dyn Environment, opts: TailOpts, call_sender: &CallSender) -> DfxResult {
    let log = env.get_logger();

    let callee_canister = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;

    let logs = canister::get_canister_logs(env, canister_id, call_sender).await?;

    info!(log, "{logs}");

    Ok(())
}
