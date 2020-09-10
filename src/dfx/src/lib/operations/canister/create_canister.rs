use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::create_waiter;

use ic_agent::ManagementCanister;
use slog::info;
use std::format;
use tokio::runtime::Runtime;

pub fn create_canister(env: &dyn Environment, canister_name: &str) -> DfxResult {
    let log = env.get_logger();
    info!(log, "Creating canister {:?}...", canister_name);

    env.get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mgr = ManagementCanister::new(
        env.get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?,
    );
    let mut runtime = Runtime::new().expect("Unable to create a runtime");

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
            let cid = runtime.block_on(mgr.create_canister(create_waiter()))?;
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
