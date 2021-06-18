use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::get_local_cid_and_candid_path;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_exponential_backoff;
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_type, print_idl_blob};

use anyhow::{anyhow, bail, Context};
use candid::{CandidType, Decode, Deserialize};
use clap::{ArgSettings, Clap};
use ic_types::principal::Principal as CanisterId;
use ic_utils::canister::{Argument, Canister};
use ic_utils::interfaces::management_canister::builders::{CanisterInstall, CanisterSettings};
use ic_utils::interfaces::management_canister::MgmtMethod;
use ic_utils::interfaces::wallet::{CallForwarder, CallResult};
use ic_utils::interfaces::Wallet;
use std::option::Option;
use std::str::FromStr;

/// Calls a method on a deployed canister.
#[derive(Clap)]
pub struct CanisterCallOpts {
    /// Specifies the name of the canister to build.
    /// You must specify either a canister name or the --all option.
    canister_name: String,

    /// Specifies the method name to call on the canister.
    method_name: String,

    /// Specifies not to wait for the result of the call to be returned by polling the replica.
    /// Instead return a response ID.
    #[clap(long)]
    r#async: bool,

    /// Sends a query request to a canister.
    #[clap(long, conflicts_with("async"))]
    query: bool,

    /// Sends an update request to a canister. This is the default if the method is not a query method.
    #[clap(long, conflicts_with("async"), conflicts_with("query"))]
    update: bool,

    /// Specifies the argument to pass to the method.
    #[clap(conflicts_with("random"))]
    argument: Option<String>,

    /// Specifies the config for generating random argument.
    #[clap(long, conflicts_with("argument"), setting = ArgSettings::AllowEmptyValues)]
    random: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    r#type: Option<String>,

    /// Specifies the format for displaying the method's return result.
    #[clap(long, conflicts_with("async"),
        possible_values(&["idl", "raw", "pp"]))]
    output: Option<String>,

    /// Specifies the amount of cycles to send on the call.
    /// Deducted from the wallet.
    #[clap(long, validator(cycle_amount_validator))]
    with_cycles: Option<String>,
}

#[derive(CandidType, Deserialize)]
struct CallIn {
    canister: CanisterId,
    method_name: String,
    #[serde(with = "serde_bytes")]
    args: Vec<u8>,
    cycles: u64,
}

async fn do_wallet_call(wallet: &Canister<'_, Wallet>, args: &CallIn) -> DfxResult<Vec<u8>> {
    let (result,): (Result<CallResult, String>,) = wallet
        .update_("wallet_call")
        .with_arg(args)
        .build()
        .call_and_wait(waiter_with_exponential_backoff())
        .await?;
    Ok(result.map_err(|err| anyhow!(err))?.r#return)
}

async fn request_id_via_wallet_call(
    wallet: &Canister<'_, Wallet>,
    canister: &Canister<'_>,
    method_name: &str,
    args: Argument,
    cycles: u64,
) -> DfxResult<ic_agent::RequestId> {
    let call_forwarder: CallForwarder<'_, '_, (CallResult,)> =
        wallet.call(canister, method_name, args, cycles);
    call_forwarder
        .call()
        .await
        .map_err(|err| anyhow!("Agent error {}", err))
}

pub fn get_effective_canister_id(
    is_management_canister: bool,
    method_name: &str,
    arg_value: &[u8],
    canister_id: CanisterId,
) -> DfxResult<CanisterId> {
    if is_management_canister {
        let method_name = MgmtMethod::from_str(method_name).map_err(|_| {
            anyhow!(
                "Attempted to call an unsupported management canister method: {}",
                method_name
            )
        })?;
        match method_name {
            MgmtMethod::CreateCanister | MgmtMethod::RawRand => {
                bail!(format!("{} can only be called via an inter-canister call. Try calling this without `--no-wallet`.",
                    method_name.as_ref()))
            }
            MgmtMethod::InstallCode => {
                let install_args = candid::Decode!(arg_value, CanisterInstall)?;
                Ok(install_args.canister_id)
            }
            MgmtMethod::UpdateSettings => {
                #[derive(CandidType, Deserialize)]
                struct In {
                    canister_id: CanisterId,
                    settings: CanisterSettings,
                }
                let in_args = candid::Decode!(arg_value, In)?;
                Ok(in_args.canister_id)
            }
            MgmtMethod::StartCanister
            | MgmtMethod::StopCanister
            | MgmtMethod::CanisterStatus
            | MgmtMethod::DeleteCanister
            | MgmtMethod::DepositCycles
            | MgmtMethod::UninstallCode
            | MgmtMethod::ProvisionalTopUpCanister => {
                #[derive(CandidType, Deserialize)]
                struct In {
                    canister_id: CanisterId,
                }
                let in_args = candid::Decode!(arg_value, In)?;
                Ok(in_args.canister_id)
            }
            MgmtMethod::ProvisionalCreateCanisterWithCycles => {
                Ok(CanisterId::management_canister())
            }
        }
    } else {
        Ok(canister_id)
    }
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterCallOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let callee_canister = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let (canister_id, maybe_candid_path) = match CanisterId::from_text(callee_canister) {
        Ok(id) => {
            if let Some(canister_name) = canister_id_store.get_name(callee_canister) {
                get_local_cid_and_candid_path(env, canister_name, Some(id))?
            } else {
                // TODO fetch candid file from remote canister
                (id, None)
            }
        }
        Err(_) => {
            let canister_id = canister_id_store.get(callee_canister)?;
            get_local_cid_and_candid_path(env, callee_canister, Some(canister_id))?
        }
    };

    let is_management_canister = canister_id == CanisterId::management_canister();

    let method_type = maybe_candid_path.and_then(|path| get_candid_type(&path, method_name));
    let is_query_method = method_type.as_ref().map(|(_, f)| f.is_query());

    let arguments = opts.argument.as_deref();
    let arg_type = opts.r#type.as_deref();
    let output_type = opts.output.as_deref();
    let is_query = if opts.r#async {
        false
    } else {
        match is_query_method {
            Some(true) => !opts.update,
            Some(false) => {
                if opts.query {
                    bail!(
                        "Invalid method call: {} is not a query method.",
                        method_name
                    );
                } else {
                    false
                }
            }
            None => opts.query,
        }
    };

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = blob_from_arguments(arguments, opts.random.as_deref(), arg_type, &method_type)?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let timeout = expiry_duration();

    // amount has been validated by cycle_amount_validator
    let cycles = opts
        .with_cycles
        .as_deref()
        .map_or(0_u64, |amount| amount.parse::<u64>().unwrap());

    if is_query {
        let blob = match call_sender {
            CallSender::SelectedId => {
                let effective_canister_id = get_effective_canister_id(
                    is_management_canister,
                    method_name,
                    &arg_value,
                    canister_id,
                )?;
                agent
                    .query(&canister_id, method_name)
                    .with_effective_canister_id(effective_canister_id)
                    .with_arg(&arg_value)
                    .call()
                    .await?
            }
            CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
                let wallet = Identity::build_wallet_canister(*wallet_id, env)?;
                do_wallet_call(
                    &wallet,
                    &CallIn {
                        canister: canister_id,
                        method_name: method_name.to_string(),
                        args: arg_value,
                        cycles,
                    },
                )
                .await?
            }
        };
        print_idl_blob(&blob, output_type, &method_type)
            .context("Invalid data: Invalid IDL blob.")?;
    } else if opts.r#async {
        let request_id = match call_sender {
            CallSender::SelectedId => {
                let effective_canister_id = get_effective_canister_id(
                    is_management_canister,
                    method_name,
                    &arg_value,
                    canister_id,
                )?;
                agent
                    .update(&canister_id, method_name)
                    .with_effective_canister_id(effective_canister_id)
                    .with_arg(&arg_value)
                    .call()
                    .await?
            }
            CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
                let wallet = Identity::build_wallet_canister(*wallet_id, env)?;
                // This is overkill, wallet.call should accept a Principal parameter
                // Why do we need to construct a Canister?
                let canister = Canister::builder()
                    .with_agent(agent)
                    .with_canister_id(canister_id)
                    .build()
                    .map_err(DfxError::from)?;
                let mut args = Argument::default();
                args.set_raw_arg(arg_value);

                request_id_via_wallet_call(&wallet, &canister, method_name, args, cycles).await?
            }
        };
        eprint!("Request ID: ");
        println!("0x{}", String::from(request_id));
    } else {
        let blob = match call_sender {
            CallSender::SelectedId => {
                let effective_canister_id = get_effective_canister_id(
                    is_management_canister,
                    method_name,
                    &arg_value,
                    canister_id,
                )?;
                agent
                    .update(&canister_id, method_name)
                    .with_effective_canister_id(effective_canister_id)
                    .with_arg(&arg_value)
                    .expire_after(timeout)
                    .call_and_wait(waiter_with_exponential_backoff())
                    .await?
            }
            CallSender::Wallet(wallet_id) | CallSender::SelectedIdWallet(wallet_id) => {
                let wallet = Identity::build_wallet_canister(*wallet_id, env)?;
                do_wallet_call(
                    &wallet,
                    &CallIn {
                        canister: canister_id,
                        method_name: method_name.to_string(),
                        args: arg_value,
                        cycles,
                    },
                )
                .await?
            }
        };

        print_idl_blob(&blob, output_type, &method_type)
            .context("Invalid data: Invalid IDL blob.")?;
    }

    Ok(())
}
