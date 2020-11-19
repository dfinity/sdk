use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::Agent;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Returns the current status of the canister on the Internet Computer network: Running, Stopping, or Stopped.
#[derive(Clap)]
#[clap(name("status"))]
pub struct CanisterStatusOpts {
    /// Specifies the name of the canister to return information for.
    /// You must specify either a canister name or the --all flag.
    canister_name: Option<String>,

    /// Returns status information for all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,
}

async fn canister_status(
    env: &dyn Environment,
    agent: &Agent,
    canister_name: &str,
    timeout: Duration,
) -> DfxResult {
    let mgr = ManagementCanister::create(agent);
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id = canister_id_store.get(canister_name)?;

    let (status,) = mgr
        .canister_status(&canister_id)
        .call_and_wait(waiter_with_timeout(timeout))
        .await?;
    info!(log, "Canister {}'s status is {}.", canister_name, status);

    Ok(())
}

pub fn exec(env: &dyn Environment, opts: CanisterStatusOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let timeout = expiry_duration();

    if let Some(canister_name) = opts.canister_name.as_deref() {
        runtime.block_on(canister_status(env, &agent, &canister_name, timeout))?;
        Ok(())
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                runtime.block_on(canister_status(env, &agent, &canister_name, timeout))?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
