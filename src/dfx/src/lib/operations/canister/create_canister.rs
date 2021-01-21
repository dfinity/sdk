use crate::lib::api_version::fetch_api_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::get_network_context;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::anyhow;
use ic_types::principal::Principal;
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

    let _ = env.get_config_or_anyhow();

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
            let network = env
                .get_network_descriptor()
                .expect("No network descriptor.");

            let mgr = ManagementCanister::create(
                env.get_agent()
                    .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
            );

            let ic_api_version = fetch_api_version(env).await?;
            let identity_name = env
                .get_selected_identity()
                .expect("No selected identity.")
                .to_string();

            let cid = if network.is_ic {
                if ic_api_version == "0.14.0" {
                    let (cid,) = mgr
                        .create_canister()
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await?;
                    cid
                } else {
                    info!(log, "Creating the canister using the wallet canister...");
                    let wallet =
                        Identity::get_or_create_wallet_canister(env, network, &identity_name, true)
                            .await?;
                    let (create_result,) = wallet
                        .wallet_create_canister(0_u64, None)
                        .call_and_wait(waiter_with_timeout(timeout))
                        .await?;
                    create_result.canister_id
                }
            } else {
                match ic_api_version.as_str() {
                    "0.14.0" => {
                        let (cid,) = mgr
                            .provisional_create_canister_with_cycles(None)
                            .call_and_wait(waiter_with_timeout(timeout))
                            .await?;
                        cid
                    }
                    _ => {
                        info!(log, "Creating the canister using the wallet canister...");
                        let wallet = Identity::get_or_create_wallet_canister(
                            env,
                            network,
                            &identity_name,
                            true,
                        )
                        .await?;
                        #[derive(candid::CandidType)]
                        struct Argument {
                            amount: Option<candid::Nat>,
                        }

                        #[derive(serde::Deserialize)]
                        struct Output {
                            canister_id: Principal,
                        }

                        let (Output { canister_id: cid },): (Output,) = wallet
                            .call_forward(
                                mgr.update_("provisional_create_canister_with_cycles")
                                    .with_arg(Argument { amount: None })
                                    .build(),
                                0_u64,
                            )?
                            .call_and_wait(waiter_with_timeout(timeout))
                            .await?;
                        cid
                    }
                }
            };
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
