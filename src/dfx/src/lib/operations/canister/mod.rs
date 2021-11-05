mod create_canister;
mod deploy_canisters;
mod install_canister;

pub use create_canister::create_canister;
pub use deploy_canisters::deploy_canisters;
pub use install_canister::install_canister;

use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings as DfxCanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::waiter::waiter_with_timeout;

use anyhow::anyhow;
use candid::utils::ArgumentDecoder;
use candid::CandidType;
use ic_types::principal::Principal as CanisterId;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::builders::CanisterSettings;
use ic_utils::interfaces::management_canister::{MgmtMethod, StatusCallResult};
use ic_utils::interfaces::ManagementCanister;
use std::path::PathBuf;
use std::time::Duration;

async fn do_management_call<A, O>(
    env: &dyn Environment,
    destination_canister: Principal,
    method: &str,
    arg: A,
    timeout: Duration,
    call_sender: &CallSender,
    cycles: u64,
) -> DfxResult<O>
where
    A: CandidType + Sync + Send,
    O: for<'de> ArgumentDecoder<'de> + Sync + Send,
{
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let mgr = ManagementCanister::create(agent);

    let out = match call_sender {
        CallSender::SelectedId => {
            mgr.update_(method)
                .with_arg(arg)
                .with_effective_canister_id(destination_canister)
                .build()
                .call_and_wait(waiter_with_timeout(timeout))
                .await?
        }
        CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
            let wallet = Identity::build_wallet_canister(*wallet_id, env)?;
            let out: O = wallet
                .call_forward(mgr.update_(method).with_arg(arg).build(), cycles)?
                .call_and_wait(waiter_with_timeout(timeout))
                .await?;
            out
        }
    };

    Ok(out)
}

pub async fn get_canister_status(
    env: &dyn Environment,
    canister_id: Principal,
    timeout: Duration,
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
        timeout,
        call_sender,
        0,
    )
    .await?;
    Ok(out)
}

pub async fn start_canister(
    env: &dyn Environment,
    canister_id: Principal,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    let _: () = do_management_call(
        env,
        canister_id,
        MgmtMethod::StartCanister.as_ref(),
        In { canister_id },
        timeout,
        call_sender,
        0,
    )
    .await?;
    Ok(())
}

pub async fn stop_canister(
    env: &dyn Environment,
    canister_id: Principal,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    let _: () = do_management_call(
        env,
        canister_id,
        MgmtMethod::StopCanister.as_ref(),
        In { canister_id },
        timeout,
        call_sender,
        0,
    )
    .await?;
    Ok(())
}

pub async fn update_settings(
    env: &dyn Environment,
    canister_id: Principal,
    settings: DfxCanisterSettings,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(candid::CandidType)]
    struct In {
        canister_id: Principal,
        settings: CanisterSettings,
    }
    let _: () = do_management_call(
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
            },
        },
        timeout,
        call_sender,
        0,
    )
    .await?;
    Ok(())
}

pub async fn uninstall_code(
    env: &dyn Environment,
    canister_id: Principal,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }
    let _: () = do_management_call(
        env,
        canister_id,
        MgmtMethod::UninstallCode.as_ref(),
        In { canister_id },
        timeout,
        call_sender,
        0,
    )
    .await?;

    Ok(())
}

pub async fn delete_canister(
    env: &dyn Environment,
    canister_id: Principal,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }
    let _: () = do_management_call(
        env,
        canister_id,
        MgmtMethod::DeleteCanister.as_ref(),
        In { canister_id },
        timeout,
        call_sender,
        0,
    )
    .await?;

    Ok(())
}

pub async fn deposit_cycles(
    env: &dyn Environment,
    canister_id: Principal,
    timeout: Duration,
    call_sender: &CallSender,
    cycles: u64,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }
    let _: () = do_management_call(
        env,
        canister_id,
        MgmtMethod::DepositCycles.as_ref(),
        In { canister_id },
        timeout,
        call_sender,
        cycles,
    )
    .await?;

    Ok(())
}

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
