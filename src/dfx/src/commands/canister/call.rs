use crate::lib::diagnosis::DiagnosedError;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::get_canister_id_and_candid_path;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::argument_from_cli::ArgumentFromCliPositionalOpt;
use crate::util::clap::parsers::cycle_amount_parser;
use crate::util::{blob_from_arguments, fetch_remote_did_file, get_candid_type, print_idl_blob};
use anyhow::bail;
use anyhow::{anyhow, Context};
use candid::Principal as CanisterId;
use candid::{CandidType, Decode, Deserialize, Principal};
use candid_parser::utils::CandidSource;
use clap::Parser;
use dfx_core::canister::build_wallet_canister;
use dfx_core::identity::CallSender;
use ic_agent::agent::CallResponse;
use ic_agent::RequestId;
use ic_utils::canister::Argument;
use ic_utils::interfaces::management_canister::builders::{CanisterInstall, CanisterSettings};
use ic_utils::interfaces::management_canister::MgmtMethod;
use ic_utils::interfaces::wallet::{CallForwarder, CallResult};
use ic_utils::interfaces::WalletCanister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::WasmResult;
use slog::warn;
use std::option::Option;
use std::path::PathBuf;
use std::str::FromStr;

/// Calls a method on a deployed canister.
#[derive(Parser)]
pub struct CanisterCallOpts {
    /// Specifies the name/id of the canister to call.
    /// You must specify either a canister or the --all option.
    canister_name: String,

    /// Specifies the method name to call on the canister.
    method_name: String,

    #[command(flatten)]
    argument_from_cli: ArgumentFromCliPositionalOpt,

    /// Specifies not to wait for the result of the call to be returned by polling the replica.
    /// Instead return a response ID.
    #[arg(long)]
    r#async: bool,

    /// Sends a query request to a canister instead of an update request.
    #[arg(long, conflicts_with("async"))]
    query: bool,

    /// Sends an update request to a canister. This is the default if the method is not a query method.
    #[arg(long, conflicts_with("async"), conflicts_with("query"))]
    update: bool,

    /// Specifies the config for generating random argument.
    #[arg(
        long,
        conflicts_with("argument"),
        conflicts_with("argument_file"),
        conflicts_with("always_assist")
    )]
    random: Option<String>,

    /// Specifies the format for displaying the method's return result.
    #[arg(long, conflicts_with("async"),
        value_parser = ["idl", "raw", "pp", "json"])]
    output: Option<String>,

    /// Specifies the amount of cycles to send on the call.
    /// Deducted from the wallet.
    /// Requires --wallet as a flag to `dfx canister`.
    #[arg(long, value_parser = cycle_amount_parser, requires("wallet"))]
    with_cycles: Option<u128>,

    /// Provide the .did file with which to decode the response.  Overrides value from dfx.json
    /// for project canisters.
    #[arg(long)]
    candid: Option<PathBuf>,

    /// Always use Candid assist when the argument types are all optional.
    #[arg(
        long,
        conflicts_with("argument"),
        conflicts_with("argument_file"),
        conflicts_with("random")
    )]
    always_assist: bool,

    /// Send request on behalf of the specified principal.
    /// This option only works for a local PocketIC instance.
    #[arg(long)]
    impersonate: Option<Principal>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
struct CallIn<TCycles = u128> {
    canister: CanisterId,
    method_name: String,
    #[serde(with = "serde_bytes")]
    args: Vec<u8>,
    cycles: TCycles,
}

async fn do_wallet_call(wallet: &WalletCanister<'_>, args: &CallIn) -> DfxResult<Vec<u8>> {
    // todo change to wallet.call when IDLValue implements ArgumentDecoder
    let builder = if wallet.version_supports_u128_cycles() {
        wallet.update("wallet_call128").with_arg(args)
    } else {
        let CallIn {
            canister,
            method_name,
            args,
            cycles,
        } = args.clone();
        let args64 = CallIn {
            canister,
            method_name,
            args,
            cycles: cycles as u64,
        };
        wallet.update("wallet_call").with_arg(args64)
    };
    let (result,): (Result<CallResult, String>,) =
        builder.build().await.context("Failed wallet call.")?;
    Ok(result.map_err(|err| anyhow!(err))?.r#return)
}

async fn request_id_via_wallet_call(
    wallet: &WalletCanister<'_>,
    canister: Principal,
    method_name: &str,
    args: Argument,
    cycles: u128,
) -> DfxResult<CallResponse<(CallResult,)>> {
    let call_forwarder: CallForwarder<'_, '_, (CallResult,)> =
        wallet.call(canister, method_name, args, cycles);
    call_forwarder
        .call()
        .await
        .map_err(|err| anyhow!("Agent error {}", err))
}

// TODO: move to ic_utils? SDKTG-302
pub fn get_effective_canister_id(
    method_name: &MgmtMethod,
    arg_value: &[u8],
) -> DfxResult<CanisterId> {
    match method_name {
        MgmtMethod::CreateCanister
        | MgmtMethod::RawRand
        | MgmtMethod::BitcoinGetBalance
        | MgmtMethod::BitcoinGetUtxos
        | MgmtMethod::BitcoinSendTransaction
        | MgmtMethod::BitcoinGetCurrentFeePercentiles
        | MgmtMethod::BitcoinGetBlockHeaders
        | MgmtMethod::EcdsaPublicKey
        | MgmtMethod::SignWithEcdsa
        | MgmtMethod::NodeMetricsHistory => Ok(CanisterId::management_canister()),
        MgmtMethod::InstallCode => {
            // TODO: Maybe this case can be merged with the following one.
            let install_args = candid::Decode!(arg_value, CanisterInstall)
                .context("Failed to decode arguments.")?;
            Ok(install_args.canister_id)
        }
        MgmtMethod::UpdateSettings => {
            // TODO: Maybe this case can be merged with the following one.
            #[derive(CandidType, Deserialize)]
            struct In {
                canister_id: CanisterId,
                settings: CanisterSettings,
            }
            let in_args = candid::Decode!(arg_value, In).context("Failed to decode arguments.")?;
            Ok(in_args.canister_id)
        }
        MgmtMethod::StartCanister
        | MgmtMethod::StopCanister
        | MgmtMethod::CanisterInfo
        | MgmtMethod::CanisterStatus
        | MgmtMethod::DeleteCanister
        | MgmtMethod::DepositCycles
        | MgmtMethod::UninstallCode
        | MgmtMethod::ProvisionalTopUpCanister
        | MgmtMethod::UploadChunk
        | MgmtMethod::ClearChunkStore
        | MgmtMethod::StoredChunks
        | MgmtMethod::FetchCanisterLogs
        | MgmtMethod::TakeCanisterSnapshot
        | MgmtMethod::LoadCanisterSnapshot
        | MgmtMethod::ListCanisterSnapshots
        | MgmtMethod::DeleteCanisterSnapshot => {
            #[derive(CandidType, Deserialize)]
            struct In {
                canister_id: CanisterId,
            }
            let in_args = candid::Decode!(arg_value, In).context("Failed to decode arguments.")?;
            Ok(in_args.canister_id)
        }
        MgmtMethod::ProvisionalCreateCanisterWithCycles => {
            // TODO: Should we use the provisional_create_canister_effective_canister_id option from main.rs?
            Ok(CanisterId::management_canister())
        }
        MgmtMethod::InstallChunkedCode => {
            #[derive(CandidType, Deserialize)]
            struct In {
                target_canister: Principal,
            }
            let in_args =
                Decode!(arg_value, In).context("Argument is not valid for InstallChunkedCode")?;
            Ok(in_args.target_canister)
        }
    }
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterCallOpts,
    mut call_sender: &CallSender,
) -> DfxResult {
    let call_sender_override = opts.impersonate.map(CallSender::Impersonate);
    if let Some(ref call_sender_override) = call_sender_override {
        call_sender = call_sender_override;
    };

    let agent = env.get_agent();
    fetch_root_key_if_needed(env).await?;

    let method_name = opts.method_name.as_str();

    let (canister_id, maybe_local_candid_path) =
        get_canister_id_and_candid_path(env, opts.canister_name.as_str())?;

    let method_type = if let Some(path) = opts.candid {
        get_candid_type(CandidSource::File(&path), method_name)
    } else if let Some(did) = fetch_remote_did_file(agent, canister_id).await {
        get_candid_type(CandidSource::Text(&did), method_name)
    } else if let Some(path) = maybe_local_candid_path {
        warn!(env.get_logger(), "DEPRECATION WARNING: Cannot fetch Candid interface from canister metadata, reading Candid interface from the local build artifact. In a future dfx release, we will only read candid interface from canister metadata.");
        warn!(
            env.get_logger(),
            r#"Please add the following to dfx.json to store local candid file into metadata:
"metadata": [
   {{
     "name": "candid:service"
   }}
]"#
        );
        get_candid_type(CandidSource::File(&path), method_name)
    } else {
        None
    };
    if method_type.is_none() {
        warn!(env.get_logger(), "Cannot fetch Candid interface for {method_name}, sending arguments with inferred types.");
    }

    let (argument_from_cli, argument_type) = opts.argument_from_cli.get_argument_and_type()?;

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = blob_from_arguments(
        Some(env),
        argument_from_cli.as_deref(),
        opts.random.as_deref(),
        argument_type.as_deref(),
        &method_type,
        false,
        opts.always_assist,
    )?;

    let effective_canister_id = if canister_id == CanisterId::management_canister() {
        let management_method = MgmtMethod::from_str(method_name).map_err(|_| {
            anyhow!(
                "Attempted to call an unsupported management canister method: {}",
                method_name
            )
        })?;

        if matches!(call_sender, CallSender::SelectedId)
            && matches!(
                management_method,
                MgmtMethod::CreateCanister
                    | MgmtMethod::RawRand
                    | MgmtMethod::BitcoinGetBalance
                    | MgmtMethod::BitcoinGetUtxos
                    | MgmtMethod::BitcoinSendTransaction
                    | MgmtMethod::BitcoinGetCurrentFeePercentiles
                    | MgmtMethod::EcdsaPublicKey
                    | MgmtMethod::SignWithEcdsa
                    | MgmtMethod::NodeMetricsHistory
            )
        {
            return Err(DiagnosedError::new(
                format!(
                    "{} can only be called by a canister, not by an external user.",
                    method_name
                ),
                format!(
                    "The easiest way to call {} externally is to proxy this call through a wallet.
Try calling this with 'dfx canister call <other arguments> (--network ic) --wallet <wallet id>'.
To figure out the id of your wallet, run 'dfx identity get-wallet (--network ic)'.",
                    method_name
                ),
            ))
            .context("Method only callable by a canister.");
        }

        get_effective_canister_id(&management_method, &arg_value)?
    } else {
        canister_id
    };

    let is_query_method = method_type.as_ref().map(|(_, f)| f.is_query());

    let output_type = opts.output.as_deref();
    let is_query = if opts.r#async {
        false
    } else {
        match is_query_method {
            Some(true) => !opts.update,
            Some(false) => {
                if opts.query {
                    return Err(DiagnosedError::new(
                        format!("{} is an update method, not a query method.", method_name),
                        "Run the command without '--query'.".to_string(),
                    ))
                    .context("Not a query method.");
                } else {
                    false
                }
            }
            None => opts.query,
        }
    };

    // amount has been validated by cycle_amount_validator
    let cycles = opts.with_cycles.unwrap_or(0);

    if call_sender == &CallSender::SelectedId && cycles != 0 {
        let explanation = "It is only possible to send cycles from a canister.";
        let action_suggestion = "To send the same function call from your wallet (a canister), run the command using 'dfx canister call <other arguments> (--network ic) --wallet <wallet id>'.\n\
        To figure out the id of your wallet, run 'dfx identity get-wallet (--network ic)'.";
        return Err(DiagnosedError::new(explanation, action_suggestion))
            .context("Function caller is not a canister.");
    }

    if is_query {
        let blob = match call_sender {
            CallSender::SelectedId => {
                let query_builder = agent
                    .query(&canister_id, method_name)
                    .with_effective_canister_id(effective_canister_id)
                    .with_arg(arg_value);
                query_builder.call().await.context("Failed query call.")?
            }
            CallSender::Impersonate(sender) => {
                let pocketic = env.get_pocketic();
                if let Some(pocketic) = pocketic {
                    let res = pocketic
                        .query_call_with_effective_principal(
                            canister_id,
                            RawEffectivePrincipal::CanisterId(
                                effective_canister_id.as_slice().to_vec(),
                            ),
                            *sender,
                            method_name,
                            arg_value,
                        )
                        .await
                        .map_err(|err| anyhow!("Failed to perform query call: {}", err))?;
                    match res {
                        WasmResult::Reply(data) => data,
                        WasmResult::Reject(err) => bail!("Canister rejected: {}", err),
                    }
                } else {
                    bail!("Impersonating sender is only supported for a local PocketIC instance.")
                }
            }
            CallSender::Wallet(wallet_id) => {
                let wallet = build_wallet_canister(*wallet_id, agent).await?;
                do_wallet_call(
                    &wallet,
                    &CallIn {
                        canister: canister_id,
                        method_name: method_name.to_string(),
                        args: arg_value,
                        cycles,
                    },
                )
                .await
                .context("Failed wallet call.")?
            }
        };
        print_idl_blob(&blob, output_type, &method_type)?;
    } else if opts.r#async {
        let call_response = match call_sender {
            CallSender::SelectedId => agent
                .update(&canister_id, method_name)
                .with_effective_canister_id(effective_canister_id)
                .with_arg(arg_value)
                .call()
                .await
                .context("Failed update call.")?
                .map(|(res, _)| res),
            CallSender::Impersonate(sender) => {
                let pocketic = env.get_pocketic();
                if let Some(pocketic) = pocketic {
                    let msg_id = pocketic
                        .submit_call_with_effective_principal(
                            canister_id,
                            RawEffectivePrincipal::CanisterId(
                                effective_canister_id.as_slice().to_vec(),
                            ),
                            *sender,
                            method_name,
                            arg_value,
                        )
                        .await
                        .map_err(|err| anyhow!("Failed to submit canister call: {}", err))?
                        .message_id;
                    CallResponse::Poll(RequestId::new(msg_id.as_slice().try_into().unwrap()))
                } else {
                    bail!("Impersonating sender is only supported for a local PocketIC instance.")
                }
            }
            CallSender::Wallet(wallet_id) => {
                let wallet = build_wallet_canister(*wallet_id, agent).await?;
                let mut args = Argument::default();
                args.set_raw_arg(arg_value);

                request_id_via_wallet_call(&wallet, canister_id, method_name, args, cycles)
                    .await
                    .context("Failed request via wallet.")?
                    .map(|(res,)| res.r#return)
            }
        };
        match call_response {
            CallResponse::Poll(request_id) => {
                eprint!("Request ID: ");
                println!("0x{}", String::from(request_id));
            }
            CallResponse::Response(response) => {
                print_idl_blob(&response, output_type, &method_type)?;
            }
        }
    } else {
        let blob = match call_sender {
            CallSender::SelectedId => agent
                .update(&canister_id, method_name)
                .with_effective_canister_id(effective_canister_id)
                .with_arg(arg_value)
                .await
                .context("Failed update call.")?,
            CallSender::Impersonate(sender) => {
                let pocketic = env.get_pocketic();
                if let Some(pocketic) = pocketic {
                    let msg_id = pocketic
                        .submit_call_with_effective_principal(
                            canister_id,
                            RawEffectivePrincipal::CanisterId(
                                effective_canister_id.as_slice().to_vec(),
                            ),
                            *sender,
                            method_name,
                            arg_value,
                        )
                        .await
                        .map_err(|err| anyhow!("Failed to submit canister call: {}", err))?;
                    let res = pocketic
                        .await_call_no_ticks(msg_id)
                        .await
                        .map_err(|err| anyhow!("Canister call failed: {}", err))?;
                    match res {
                        WasmResult::Reply(data) => data,
                        WasmResult::Reject(err) => bail!("Canister rejected: {}", err),
                    }
                } else {
                    bail!("Impersonating sender is only supported for a local PocketIC instance.")
                }
            }
            CallSender::Wallet(wallet_id) => {
                let wallet = build_wallet_canister(*wallet_id, agent).await?;
                do_wallet_call(
                    &wallet,
                    &CallIn {
                        canister: canister_id,
                        method_name: method_name.to_string(),
                        args: arg_value,
                        cycles,
                    },
                )
                .await
                .context("Failed to do wallet call.")?
            }
        };
        print_idl_blob(&blob, output_type, &method_type)?;
    }

    Ok(())
}
