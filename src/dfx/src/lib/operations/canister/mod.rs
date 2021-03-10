mod create_canister;
mod deploy_canisters;
mod install_canister;

pub use create_canister::create_canister;
pub use deploy_canisters::deploy_canisters;
pub use install_canister::install_canister;

use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::{wallet_for_call_sender, CallSender};
use crate::lib::waiter::waiter_with_timeout;
use anyhow::anyhow;
use candid::de::ArgumentDecoder;
use candid::CandidType;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::CanisterStatus;
use ic_utils::interfaces::ManagementCanister;
use serde::Deserialize;
use std::time::Duration;

async fn do_management_call<A, O>(
    env: &dyn Environment,
    method: &str,
    arg: A,
    timeout: Duration,
    call_sender: &CallSender,
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
                .build()
                .call_and_wait(waiter_with_timeout(timeout))
                .await?
        }
        CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
            let wallet = wallet_for_call_sender(env, call_sender, wallet_id).await?;
            let out: O = wallet
                .call_forward(mgr.update_(method).with_arg(arg).build(), 0)?
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
) -> DfxResult<CanisterStatus> {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    #[derive(Deserialize)]
    struct Out {
        status: CanisterStatus,
    }

    let (out,): (Out,) = do_management_call(
        env,
        "canister_status",
        In { canister_id },
        timeout,
        call_sender,
    )
    .await?;
    Ok(out.status)
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
        "start_canister",
        In { canister_id },
        timeout,
        call_sender,
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
        "stop_canister",
        In { canister_id },
        timeout,
        call_sender,
    )
    .await?;
    Ok(())
}

pub async fn set_controller(
    env: &dyn Environment,
    canister_id: Principal,
    new_controller: Principal,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
        new_controller: Principal,
    }

    let _: () = do_management_call(
        env,
        "set_controller",
        In {
            canister_id,
            new_controller,
        },
        timeout,
        call_sender,
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
        "delete_canister",
        In { canister_id },
        timeout,
        call_sender,
    )
    .await?;

    Ok(())
}
