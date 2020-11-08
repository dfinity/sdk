use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::Context;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::format;
use std::time::Duration;
use tokio::runtime::Runtime;

pub fn create_canister(env: &dyn Environment, canister_name: &str, timeout: Duration) -> DfxResult {
    let log = env.get_logger();
    info!(log, "Creating canister {:?}...", canister_name);

    env.get_config()
        .context("Command must be run in a project directory (with a dfx.json file).")?;

    let mut canister_id_store = CanisterIdStore::for_env(env)?;

    let network_name = get_network_context()?;

    let non_default_network = if network_name == "local" {
        format!("")
    } else {
        format!("on network {:?} ", network_name)
    };

    match canister_id_store.find(&canister_name) {
        Some(canister_id) => {
            info!(
                log,
                "{:?} canister was already created {}and has canister id: {:?}",
                canister_name,
                non_default_network,
                canister_id.to_text()
            );
            Ok(())
        }
        None => {
            let mgr = ManagementCanister::create(
                env.get_agent()
                   .context("Command must be run in a project directory (with a dfx.json file).")?;
            );

            let mut runtime = Runtime::new().expect("Unable to create a runtime");
            let (cid,) = runtime.block_on(
                mgr.create_canister()
                    .call_and_wait(waiter_with_timeout(timeout)),
            )?;
            let canister_id = cid.to_text();
            info!(
                log,
                "{:?} canister created {}with canister id: {:?}",
                canister_name,
                non_default_network,
                canister_id
            );
            canister_id_store.add(&canister_name, canister_id)
        }
    }?;

    Ok(())
}
