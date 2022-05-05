use anyhow::{anyhow, Context, Error};
use candid::{CandidType, Deserialize, Principal};
use ic_agent::Identity as _;
use ic_utils::{
    call::AsyncCall,
    interfaces::{
        management_canister::{
            builders::{CanisterSettings, InstallMode},
            StatusCallResult,
        },
        ManagementCanister, WalletCanister,
    },
    Argument,
};

use crate::{
    lib::waiter::waiter_with_timeout,
    util::{assets::wallet_wasm, expiry_duration},
};

use super::{
    environment::Environment,
    error::DfxResult,
    identity::{Identity, IdentityManager},
    models::canister_id_store::CanisterIdStore,
    network::network_descriptor::NetworkDescriptor,
};

pub async fn migrate(env: &dyn Environment, network: &NetworkDescriptor, fix: bool) -> DfxResult {
    let agent = env
        .get_agent()
        .expect("Could not get agent from environment");
    let mut mgr = IdentityManager::new(env)?;
    let ident = mgr.instantiate_selected_identity()?;
    let wallet = Identity::wallet_canister_id(env, network, ident.name())
        .map_err(|_| anyhow!("No wallet found; nothing to do"))?;
    let wallet = WalletCanister::create(agent, wallet)
        .await
        .context("No accessible wallet found; nothing to do")?;
    let mgmt = ManagementCanister::create(agent);
    if !wallet.version_supports_u128_cycles() {
        if fix {
            println!("Upgrading wallet... ");
            let wasm = wallet_wasm(env.get_logger())?;
            mgmt.install_code(wallet.canister_id_(), &wasm)
                .with_mode(InstallMode::Upgrade)
                .call_and_wait(waiter_with_timeout(expiry_duration()))
                .await
                .context("Could not upgrade wallet")?;
        } else {
            println!("The wallet is outdated; run `dfx wallet upgrade`");
        }
    }
    let store = CanisterIdStore::for_env(env)?;
    for (name, _) in &store.ids {
        if let Some(id) = store.find(name) {
            if let Err(_) = mgmt
                .canister_status(&id)
                .call_and_wait(waiter_with_timeout(expiry_duration()))
                .await
            {
                if fix {
                    println!(
                        "Adding the {ident} identity to canister {name}'s controllers...",
                        ident = ident.name()
                    );
                    #[derive(CandidType, Deserialize)]
                    struct In {
                        canister_id: Principal,
                    }
                    let (status,): (StatusCallResult,) = wallet
                        .call(
                            Principal::management_canister(),
                            "canister_status",
                            Argument::from_candid((In { canister_id: id },)),
                            0,
                        )
                        .call_and_wait(waiter_with_timeout(expiry_duration()))
                        .await
                        .context("Could not access canister through wallet")?;
                    let mut controllers = status.settings.controllers;
                    controllers.push(ident.sender().map_err(Error::msg)?);
                    #[derive(CandidType, Deserialize)]
                    struct In2 {
                        canister_id: Principal,
                        settings: CanisterSettings,
                    }
                    wallet
                        .call(
                            Principal::management_canister(),
                            "update_settings",
                            Argument::from_candid((In2 {
                                canister_id: id,
                                settings: CanisterSettings {
                                    controllers: Some(controllers),
                                    compute_allocation: None,
                                    freezing_threshold: None,
                                    memory_allocation: None,
                                },
                            },)),
                            0,
                        )
                        .call_and_wait(waiter_with_timeout(expiry_duration()))
                        .await
                        .context("Could not update canister settings")?;
                } else {
                    println!("Canister {name} is outdated; run `dfx canister update-settings` with the --add-controller flag")
                }
            }
        }
    }
    Ok(())
}
