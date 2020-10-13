use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::IdentityManager;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::waiter_with_timeout;

use candid::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use slog::info;
use std::format;
use std::time::Duration;

pub async fn create_canister(
    env: &dyn Environment,
    canister_name: &str,
    timeout: Duration,
) -> DfxResult {
    let log = env.get_logger();
    info!(log, "Creating canister {:?}...", canister_name);

    env.get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

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
            // Get the wallet canister.
            let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
            let network = env.get_network_descriptor().expect("no network descriptor");
            let wallet = identity.get_wallet(env, network, true).await?;

            let mgr = ManagementCanister::create(
                env.get_agent()
                    .ok_or(DfxError::CommandMustBeRunInAProject)?,
            );

            info!(log, "Creating the canister using the wallet canister...");
            #[derive(serde::Deserialize)]
            struct Output {
                canister_id: Principal,
            }

            let (Output { canister_id: cid },): (Output,) = wallet
                .call_forward(mgr.update_("create_canister").build(), 0)?
                .call_and_wait(waiter_with_timeout(timeout))
                .await?;
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
