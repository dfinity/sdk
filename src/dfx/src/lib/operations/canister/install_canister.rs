use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::installers::assets::post_install_store_assets;
use crate::lib::named_canister;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::Context;
use ic_agent::Agent;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::{ComputeAllocation, InstallMode, MemoryAllocation};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Canister;
use serde_bytes::ByteBuf;
use slog::info;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub async fn install_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_info: &CanisterInfo,
    args: &[u8],
    compute_allocation: Option<ComputeAllocation>,
    mode: InstallMode,
    memory_allocation: Option<MemoryAllocation>,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    let network = env.get_network_descriptor().unwrap();
    if named_canister::get_ui_canister_id(&network).is_none() {
        named_canister::install_ui_canister(env, &network, None).await?;
    }

    let mgr = ManagementCanister::create(agent);
    let log = env.get_logger();
    let canister_id = canister_info.get_canister_id().context(format!(
        "Cannot find build output for canister '{}'. Did you forget to run `dfx build`?",
        canister_info.get_name().to_owned()
    ))?;
    info!(
        log,
        "Installing code for canister {}, with canister_id {}",
        canister_info.get_name(),
        canister_id,
    );

    let wasm_path = canister_info
        .get_output_wasm_path()
        .expect("Cannot get WASM output path.");
    let wasm_module = std::fs::read(wasm_path)?;

    match call_sender {
        CallSender::SelectedId => {
            let install_builder = mgr
                .install_code(&canister_id, &wasm_module)
                .with_raw_arg(args.to_vec())
                .with_mode(mode);
            let install_builder = if let Some(ca) = compute_allocation {
                install_builder.with_compute_allocation(ca)
            } else {
                install_builder
            };
            let install_builder = if let Some(ma) = memory_allocation {
                install_builder.with_memory_allocation(ma)
            } else {
                install_builder
            };
            install_builder
                .build()?
                .call_and_wait(waiter_with_timeout(timeout))
                .await?;
        }
        CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
            let wallet = Identity::build_wallet_canister(wallet_id.clone(), env)?;

            #[derive(candid::CandidType)]
            struct CanisterInstall {
                mode: InstallMode,
                canister_id: Principal,
                wasm_module: ByteBuf,
                arg: ByteBuf,
                compute_allocation: Option<candid::Nat>,
                memory_allocation: Option<candid::Nat>,
            }

            let install_args = CanisterInstall {
                mode,
                canister_id: canister_id.clone(),
                wasm_module: ByteBuf::from(wasm_module),
                arg: ByteBuf::from(args.to_vec()),
                compute_allocation: compute_allocation.map(|x| candid::Nat::from(u8::from(x))),
                memory_allocation: memory_allocation.map(|x| candid::Nat::from(u64::from(x))),
            };
            wallet
                .call_forward(
                    mgr.update_("install_code").with_arg(install_args).build(),
                    0,
                )?
                .call_and_wait(waiter_with_timeout(timeout))
                .await?;
        }
    }

    if canister_info.get_type() == "assets" {
        match call_sender {
            CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
                let wallet = Identity::build_wallet_canister(wallet_id.clone(), env)?;
                let identity_name = env.get_selected_identity().expect("No selected identity.");
                info!(
                    log,
                    "Authorizing our identity ({}) to the asset canister...", identity_name
                );
                let canister = Canister::builder()
                    .with_agent(agent)
                    .with_canister_id(canister_id.clone())
                    .build()
                    .unwrap();
                let self_id = env
                    .get_selected_identity_principal()
                    .expect("Selected identity not instantiated.");
                // Before storing assets, make sure the DFX principal is in there first.
                wallet
                    .call_forward(canister.update_("authorize").with_arg(self_id).build(), 0)?
                    .call_and_wait(waiter_with_timeout(timeout))
                    .await?;
            }
            _ => (),
        };

        info!(log, "Uploading assets to asset canister...");
        post_install_store_assets(&canister_info, &agent, timeout).await?;
    }

    Ok(())
}
