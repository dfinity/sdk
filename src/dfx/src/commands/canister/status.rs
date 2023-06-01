use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use dfx_core::identity::CallSender;

use candid::Principal;
use clap::Parser;
use fn_error_context::context;
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

    let mut controllers: Vec<_> = status
        .settings
        .controllers
        .iter()
        .map(Principal::to_text)
        .collect();
    controllers.sort();

    info!(log, "Canister status call result for {}.\nStatus: {}\nControllers: {}\nMemory allocation: {}\nCompute allocation: {}\nFreezing threshold: {}\nMemory Size: {:?}\nBalance: {} Cycles\nModule hash: {}",
        canister,
        status.status,
        controllers.join(" "),
        status.settings.memory_allocation,
        status.settings.compute_allocation,
        status.settings.freezing_threshold,
        status.memory_size,
        status.cycles,
        status.module_hash.map_or_else(|| "None".to_string(), |v| format!("0x{}", hex::encode(v)))
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
