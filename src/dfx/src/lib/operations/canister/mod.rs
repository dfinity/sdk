pub(crate) mod create_canister;
pub(crate) mod deploy_canisters;
pub(crate) mod install_canister;
pub mod motoko_playground;

pub use create_canister::create_canister;
use ic_utils::interfaces::management_canister::Snapshot;
pub use install_canister::install_wallet;

use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings as DfxCanisterSettings;
use anyhow::{anyhow, bail, Context};
use candid::utils::ArgumentDecoder;
use candid::CandidType;
use candid::Principal as CanisterId;
use candid::Principal;
use candid::{decode_args, encode_args};
use dfx_core::canister::build_wallet_canister;
use dfx_core::config::model::dfinity::Config;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_utils::call::SyncCall;
use ic_utils::interfaces::management_canister::builders::CanisterSettings;
use ic_utils::interfaces::management_canister::{
    FetchCanisterLogsResponse, MgmtMethod, StatusCallResult,
};
use ic_utils::interfaces::ManagementCanister;
use ic_utils::Argument;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::WasmResult;
use std::collections::HashSet;
use std::path::PathBuf;

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
    let agent = env.get_agent();
    let out = match call_sender {
        CallSender::SelectedId => {
            let mgr = ManagementCanister::create(agent);

            mgr.update(method)
                .with_arg(arg)
                .with_effective_canister_id(destination_canister)
                .build()
                .await
                .context("Update call (without wallet) failed.")?
        }
        CallSender::Impersonate(sender) => {
            let pocketic = env.get_pocketic();
            if let Some(pocketic) = pocketic {
                let msg_id = pocketic
                    .submit_call_with_effective_principal(
                        Principal::management_canister(),
                        RawEffectivePrincipal::CanisterId(destination_canister.as_slice().to_vec()),
                        *sender,
                        method,
                        encode_args((arg,)).unwrap(),
                    )
                    .await
                    .map_err(|err| anyhow!("Failed to submit management canister call: {}", err))?;
                let res = pocketic
                    .await_call_no_ticks(msg_id)
                    .await
                    .map_err(|err| anyhow!("Management canister call failed: {}", err))?;
                match res {
                    WasmResult::Reply(data) => decode_args(&data)
                        .context("Could not decode management canister response.")?,
                    WasmResult::Reject(err) => bail!("Management canister call rejected: {}", err),
                }
            } else {
                bail!("Impersonating sender is only supported for a local PocketIC instance.")
            }
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
                .await
                .context("Update call using wallet failed.")?;
            out
        }
    };

    Ok(out)
}

#[context(
    "Failed to query call function '{}' regarding canister '{}'.",
    method,
    destination_canister
)]
async fn do_management_query_call<A, O>(
    env: &dyn Environment,
    destination_canister: Principal,
    method: &str,
    arg: A,
    call_sender: &CallSender,
) -> DfxResult<O>
where
    A: CandidType + Sync + Send,
    O: for<'de> ArgumentDecoder<'de> + Sync + Send,
{
    let agent = env.get_agent();
    let out = match call_sender {
        CallSender::SelectedId => {
            let mgr = ManagementCanister::create(agent);

            mgr.query(method)
                .with_arg(arg)
                .with_effective_canister_id(destination_canister)
                .build()
                .call()
                .await
                .context("Query call (without wallet) failed.")?
        }
        CallSender::Impersonate(_) => {
            unreachable!(
                "Impersonating sender in management canister query calls in not supported."
            )
        }
        CallSender::Wallet(wallet_id) => {
            let wallet = build_wallet_canister(*wallet_id, agent).await?;
            let out: O = wallet
                .query(method)
                .with_arg(arg)
                .with_effective_canister_id(Principal::management_canister())
                .build()
                .call()
                .await
                .context("Query call using wallet failed.")?;
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

#[context("Failed to get canister logs of {}.", canister_id)]
pub async fn get_canister_logs(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
) -> DfxResult<FetchCanisterLogsResponse> {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }

    let (out,): (FetchCanisterLogsResponse,) = do_management_query_call(
        env,
        canister_id,
        MgmtMethod::FetchCanisterLogs.as_ref(),
        In { canister_id },
        call_sender,
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
            settings: settings.into(),
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

/// Get the canister id and the path to the candid file for the given canister.
/// The argument `canister` can be either a canister id or a canister name.
pub fn get_canister_id_and_candid_path(
    env: &dyn Environment,
    canister: &str,
) -> DfxResult<(CanisterId, Option<PathBuf>)> {
    let canister_id_store = env.get_canister_id_store()?;
    let (canister_name, canister_id) = if let Ok(id) = Principal::from_text(canister) {
        if let Some(canister_name) = canister_id_store.get_name(canister) {
            (canister_name.to_string(), id)
        } else {
            return Ok((id, None));
        }
    } else {
        (canister.to_string(), canister_id_store.get(canister)?)
    };
    let config = env.get_config_or_anyhow()?;
    let candid_path = match CanisterInfo::load(&config, &canister_name, Some(canister_id)) {
        Ok(info) => Some(info.get_output_idl_path().to_path_buf()),
        // In a rare case that the canister was deployed and then removed from dfx.json,
        // the canister_id_store can still resolve the canister id from the canister name.
        // In such case, technically, we are still able to call the canister.
        Err(_) => None,
    };
    Ok((canister_id, candid_path))
}

pub fn add_canisters_with_ids(
    canister_names: &[String],
    env: &dyn Environment,
    config: &Config,
) -> Vec<String> {
    let mut canister_names: HashSet<_> = canister_names.iter().cloned().collect();

    canister_names.extend(all_project_canisters_with_ids(env, config));

    canister_names.into_iter().collect()
}

pub fn all_project_canisters_with_ids(env: &dyn Environment, config: &Config) -> Vec<String> {
    env.get_canister_id_store()
        .map(|store| {
            config
                .get_config()
                .canisters
                .as_ref()
                .map(|canisters| {
                    canisters
                        .keys()
                        .filter(|canister| store.get(canister).is_ok())
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

#[context("Failed to take snapshot in canister {canister_id}")]
pub async fn take_canister_snapshot(
    env: &dyn Environment,
    canister_id: Principal,
    replace_snapshot: Option<&[u8]>,
    call_sender: &CallSender,
) -> DfxResult<Snapshot> {
    #[derive(CandidType)]
    struct In<'a> {
        canister_id: Principal,
        replace_snapshot: Option<&'a [u8]>,
    }
    let (snapshot,) = do_management_call(
        env,
        canister_id,
        MgmtMethod::TakeCanisterSnapshot.as_ref(),
        &In {
            canister_id,
            replace_snapshot,
        },
        call_sender,
        0,
    )
    .await?;
    Ok(snapshot)
}

#[context(
    "Failed to load snapshot {} in canister {canister_id}",
    hex::encode(snapshot_id)
)]
pub async fn load_canister_snapshot(
    env: &dyn Environment,
    canister_id: Principal,
    snapshot_id: &[u8],
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In<'a> {
        canister_id: Principal,
        snapshot_id: &'a [u8],
        sender_canister_version: Option<u64>,
    }
    do_management_call(
        env,
        canister_id,
        MgmtMethod::LoadCanisterSnapshot.as_ref(),
        &In {
            canister_id,
            snapshot_id,
            sender_canister_version: None,
        },
        call_sender,
        0,
    )
    .await?;
    Ok(())
}

#[context("Failed to list snapshots in canister {canister_id}")]
pub async fn list_canister_snapshots(
    env: &dyn Environment,
    canister_id: Principal,
    call_sender: &CallSender,
) -> DfxResult<Vec<Snapshot>> {
    #[derive(CandidType)]
    struct In {
        canister_id: Principal,
    }
    let (snapshots,) = do_management_call(
        env,
        canister_id,
        MgmtMethod::ListCanisterSnapshots.as_ref(),
        &In { canister_id },
        call_sender,
        0,
    )
    .await?;
    Ok(snapshots)
}

#[context(
    "Failed to delete canister snapshot {} in canister {canister_id}",
    hex::encode(snapshot_id)
)]
pub async fn delete_canister_snapshot(
    env: &dyn Environment,
    canister_id: Principal,
    snapshot_id: &[u8],
    call_sender: &CallSender,
) -> DfxResult {
    #[derive(CandidType)]
    struct In<'a> {
        canister_id: Principal,
        snapshot_id: &'a [u8],
    }
    do_management_call(
        env,
        canister_id,
        MgmtMethod::DeleteCanisterSnapshot.as_ref(),
        &In {
            canister_id,
            snapshot_id,
        },
        call_sender,
        0,
    )
    .await?;
    Ok(())
}
