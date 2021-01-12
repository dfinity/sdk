use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::Agent;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::time::Duration;

/// Starts a canister on the Internet Computer network.
#[derive(Clap)]
pub struct CanisterStartOpts {
    /// Specifies the name of the canister to start. You must specify either a canister name or the --all flag.
    canister_name: Option<String>,

    /// Starts all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,
}

async fn start_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_name: &str,
    timeout: Duration,
) -> DfxResult {
    let mgr = ManagementCanister::create(agent);
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id = canister_id_store.get(canister_name)?;

    info!(
        log,
        "Starting code for canister {}, with canister_id {}",
        canister_name,
        canister_id.to_text(),
    );

    mgr.start_canister(&canister_id)
        .call_and_wait(waiter_with_timeout(timeout))
        .await
        .map_err(DfxError::from)?;

    Ok(())
}

pub async fn exec(env: &dyn Environment, opts: CanisterStartOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    fetch_root_key_if_needed(env).await?;

    let timeout = expiry_duration();

    if let Some(canister_name) = opts.canister_name.as_deref() {
        start_canister(env, &agent, &canister_name, timeout).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                start_canister(env, &agent, &canister_name, timeout).await?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
