use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use slog::info;

const SECONDS_PER_DAY: u128 = 24 * 60 * 60;

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

    let reserved_cycles_limit = if let Some(limit) = status.settings.reserved_cycles_limit {
        format!("{} Cycles", limit)
    } else {
        "Not Set".to_string()
    };

    let freezing_limit_cycles = (status.idle_cycles_burned_per_day
        * status.settings.freezing_threshold.clone()
        / SECONDS_PER_DAY)
        - status.reserved_cycles.clone();

    info!(log, "Canister status call result for {}.\nStatus: {}\nControllers: {}\nMemory allocation: {}\nCompute allocation: {}\nFreezing threshold in seconds: {}\nFreezing limit in cycles: {}\nMemory Size: {:?}\nBalance: {} Cycles\nReserved: {} Cycles\nReserved Cycles Limit: {}\nModule hash: {}",
        canister,
        status.status,
        controllers.join(" "),
        status.settings.memory_allocation,
        status.settings.compute_allocation,
        status.settings.freezing_threshold,
        freezing_limit_cycles,
        status.memory_size,
        status.cycles,
        status.reserved_cycles,
        reserved_cycles_limit,
        status.module_hash.map_or_else(|| "None".to_string(), |v| format!("0x{}", hex::encode(v)))
    );

    if status.cycles < freezing_limit_cycles {
        info!(log, "\nCanister {} is frozen. Please top it up with at least {} cycles to unfreeze it. Then top it up with enough cycles to account for processing calls.", canister, freezing_limit_cycles - status.cycles);
    }

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
