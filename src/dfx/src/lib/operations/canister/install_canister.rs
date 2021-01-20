use crate::lib::api_version::fetch_api_version;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity as DfxIdentity;
use crate::lib::installers::assets::post_install_store_assets;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::Context;
use ic_agent::Agent;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::{ComputeAllocation, InstallMode, MemoryAllocation};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Canister;
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
) -> DfxResult {
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

    let identity_name = env
        .get_selected_identity()
        .expect("no selected identity")
        .to_string();
    let network = env.get_network_descriptor().expect("no network descriptor");

    let ic_api_version = fetch_api_version(env).await?;

    if ic_api_version == "0.14.0" {
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
    } else {
        #[derive(candid::CandidType)]
        struct CanisterInstall {
            mode: InstallMode,
            canister_id: Principal,
            wasm_module: Vec<u8>,
            arg: Vec<u8>,
            compute_allocation: Option<candid::Nat>,
            memory_allocation: Option<candid::Nat>,
        }

        let install_args = CanisterInstall {
            mode,
            canister_id: canister_id.clone(),
            wasm_module,
            arg: args.to_vec(),
            compute_allocation: compute_allocation.map(|x| candid::Nat::from(u8::from(x))),
            memory_allocation: memory_allocation.map(|x| candid::Nat::from(u64::from(x))),
        };
        let wallet = DfxIdentity::get_wallet_canister(env, network, &identity_name).await?;

        wallet
            .call_forward(
                mgr.update_("install_code").with_arg(install_args).build(),
                0,
            )?
            .call_and_wait(waiter_with_timeout(timeout))
            .await?;
    }

    if canister_info.get_type() == "assets" {
        if ic_api_version != "0.14.0" {
            let wallet = DfxIdentity::get_wallet_canister(env, network, &identity_name).await?;
            let self_id = env
                .get_selected_identity_principal()
                .expect("selected identity not instantiated");
            let identity_name = env
                .get_selected_identity()
                .expect("selected identity not instantiated");
            info!(
                log,
                "Authorizing our identity ({}) to the asset canister...", identity_name
            );
            let canister = Canister::builder()
                .with_agent(agent)
                .with_canister_id(canister_id.clone())
                .build()
                .unwrap();

            // Before storing assets, make sure the DFX principal is in there first.
            wallet
                .call_forward(canister.update_("authorize").with_arg(self_id).build(), 0)?
                .call_and_wait(waiter_with_timeout(timeout))
                .await?;
        }

        info!(log, "Uploading assets to asset canister...");
        post_install_store_assets(&canister_info, &agent, timeout).await?;
    }

    Ok(())
}
