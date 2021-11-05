use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::expiry_duration;

use clap::Clap;
use ic_types::Principal;
use slog::info;
use std::time::Duration;

/// Returns the current status of the canister on the Internet Computer network: Running, Stopping, or Stopped.
#[derive(Clap)]
pub struct CanisterStatusOpts {
    /// Specifies the name of the canister to return information for.
    /// You must specify either a canister name or the --all flag.
    canister: Option<String>,

    /// Returns status information for all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,
}

async fn canister_status(
    env: &dyn Environment,
    canister: &str,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    let status = canister::get_canister_status(env, canister_id, timeout, call_sender).await?;

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
    let config = env.get_config_or_anyhow()?;

    fetch_root_key_if_needed(env).await?;
    let timeout = expiry_duration();

    if let Some(canister) = opts.canister.as_deref() {
        canister_status(env, canister, timeout, call_sender).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                canister_status(env, canister, timeout, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
