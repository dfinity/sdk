use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::progress_bar::ProgressBar;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::create_waiter;

use ic_agent::ManagementCanister;
use std::format;
use tokio::runtime::Runtime;

pub fn create_canister(env: &dyn Environment, canister_name: &str) -> DfxResult {
    let message = format!("Creating canister {:?}...", canister_name);
    let b = ProgressBar::new_spinner(&message);

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
            let message = format!(
                "{:?} canister was already created {}and has canister id: {:?}",
                canister_name,
                non_default_network,
                canister_id.to_text()
            );
            b.finish_with_message(&message);
            Ok(())
        }
        None => {
            let cid = runtime.block_on(mgr.create_canister(create_waiter()))?;
            let canister_id = cid.to_text();
            let message = format!(
                "{:?} canister created {}with canister id: {:?}",
                canister_name, non_default_network, canister_id
            );
            b.finish_with_message(&message);
            canister_id_store.add(&canister_name, canister_id)
        }
    }?;

    Ok(())
}
