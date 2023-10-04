mod create_canister;
pub(crate) mod deploy_canisters;
pub(crate) mod install_canister;
pub use create_canister::create_canister;

use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings as DfxCanisterSettings;
use anyhow::{anyhow, bail, Context};
use candid::utils::ArgumentDecoder;
use candid::CandidType;
use candid::Principal as CanisterId;
use candid::Principal;
pub use deploy_canisters::deploy_canisters;
pub use deploy_canisters::{
    DeployMode, Funding, ICPFunding, ICPFundingRetry, ICPFundingRetryPhase,
};
use dfx_core::canister::build_wallet_canister;
pub use dfx_core::canister::install_canister_wasm;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::builders::CanisterSettings;
use ic_utils::interfaces::management_canister::{MgmtMethod, StatusCallResult};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Argument;
pub use install_canister::{install_canister, install_wallet};
use std::path::PathBuf;

pub mod motoko_playground;

#[context(
    "Failed to call update function '{}' regarding canister '{}'.",
    method,
    destination_canister
)]
async fn do_management_call<A, O>(
    env: &dyn Environment,
    destination_canister: Principal,
    method: &str,
    arg: A,
    call_sender: &CallSender,
    cycles: u128,
) -> DfxResult<O>
where
    A: CandidType + Sync + Send,
    O: for<'de> ArgumentDecoder<'de> + Sync + Send,
{
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let out = match call_sender {
        CallSender::SelectedId => {
            let mgr = ManagementCanister::create(agent);

            mgr.update_(method)
                .with_arg(arg)
                .with_effective_canister_id(destination_canister)
                .build()
                .call_and_wait()
                .await
                .context("Update call (without wallet) failed.")?
        }
        CallSender::Wallet(wallet_id) => {
            let wallet = build_wallet_canister(*wallet_id, agent).await?;
            let out: O = wallet
                .call(
                    Principal::management_canister(),
                    method,
                    Argument::from_candid((arg,)),
                    cycles,
                )
                .call_and_wait()
                .await
                .context("Update call using wallet failed.")?;
            out
        }
    };

    Ok(out)
}

#[context("Failed to get canister status of {}.", canister_id)]
pub async fn get_canister_status(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
) -> DfxResult<StatusCallResult> {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    let (out,): (StatusCallResult,) = do_management_call(
        env,
        canister_id,
        MgmtMethod::CanisterStatus.as_ref(),
        In { canister_id },
        call_sender,
        0,
    )
    .await?;
    Ok(out)
}

#[context("Failed to start canister {}.", canister_id)]
pub async fn start_canister(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    do_management_call(
        env,
        canister_id,
        MgmtMethod::StartCanister.as_ref(),
        In { canister_id },
        call_sender,
        0,
    )
    .await?;
    Ok(())
}

#[context("Failed to stop canister {}.", canister_id)]
pub async fn stop_canister(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
) -> DfxResult {
    if env.get_network_descriptor().is_playground() {
        bail!("Canisters borrowed from a playground cannot be stopped.");
    }

    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    do_management_call(
        env,
        canister_id,
        MgmtMethod::StopCanister.as_ref(),
        In { canister_id },
        call_sender,
        0,
    )
    .await?;
    Ok(())
}

#[context("Failed to update settings for {}.", canister_id)]
pub async fn update_settings(
    env: &dyn Environment,
    canister_id: Principal,
    settings: DfxCanisterSettings,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(candid::CandidType)]
    struct In {
        canister_id: Principal,
        settings: CanisterSettings,
    }
    do_management_call(
        env,
        canister_id,
        MgmtMethod::UpdateSettings.as_ref(),
        In {
            canister_id,
            settings: CanisterSettings {
                controllers: settings.controllers,
                compute_allocation: settings
                    .compute_allocation
                    .map(u8::from)
                    .map(candid::Nat::from),
                memory_allocation: settings
                    .memory_allocation
                    .map(u64::from)
                    .map(candid::Nat::from),
                freezing_threshold: settings
                    .freezing_threshold
                    .map(u64::from)
                    .map(candid::Nat::from),
                reserved_cycles_limit: settings
                    .reserved_cycles_limit
                    .map(u128::from)
                    .map(candid::Nat::from),
            },
        },
        call_sender,
        0,
    )
    .await?;
    Ok(())
}

#[context("Failed to uninstall code for {}.", canister_id)]
pub async fn uninstall_code(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }
    do_management_call(
        env,
        canister_id,
        MgmtMethod::UninstallCode.as_ref(),
        In { canister_id },
        call_sender,
        0,
    )
    .await?;

    Ok(())
}

#[context("Failed to delete {}.", canister_id)]
pub async fn delete_canister(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }
    do_management_call(
        env,
        canister_id,
        MgmtMethod::DeleteCanister.as_ref(),
        In { canister_id },
        call_sender,
        0,
    )
    .await?;

    Ok(())
}

#[context("Failed to deposit {} cycles into {}.", cycles, canister_id)]
pub async fn deposit_cycles(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
    cycles: u128,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }
    do_management_call(
        env,
        canister_id,
        MgmtMethod::DepositCycles.as_ref(),
        In { canister_id },
        call_sender,
        cycles,
    )
    .await?;

    Ok(())
}

/// Can only run this locally, not on the real IC.
/// Conjures cycles from nothing and deposits them in the selected canister.
#[context(
    "Failed provisional deposit of {} cycles to canister {}.",
    cycles,
    canister_id
)]
pub async fn provisional_deposit_cycles(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
    cycles: u128,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
        amount: u128,
    }
    do_management_call(
        env,
        canister_id,
        MgmtMethod::ProvisionalTopUpCanister.as_ref(),
        In {
            canister_id,
            amount: cycles,
        },
        call_sender,
        0,
    )
    .await?;

    Ok(())
}

#[context(
    "Failed to get canister id and path to its candid definitions for '{}'.",
    canister_name
)]
pub fn get_local_cid_and_candid_path(
    env: &dyn Environment,
    canister_name: &str,
    maybe_canister_id: Option<CanisterId>,
) -> DfxResult<(CanisterId, Option<PathBuf>)> {
    let config = env.get_config_or_anyhow()?;
    let canister_info = CanisterInfo::load(&config, canister_name, maybe_canister_id)?;
    Ok((
        canister_info.get_canister_id()?,
        canister_info.get_output_idl_path(),
    ))
}
