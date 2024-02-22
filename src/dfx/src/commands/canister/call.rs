use crate::lib::diagnosis::DiagnosedError;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::get_local_cid_and_candid_path;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::{cycle_amount_parser, file_or_stdin_parser};
use crate::util::{
    arguments_from_file, blob_from_arguments, fetch_remote_did_file, get_candid_type,
    print_idl_blob,
};
use anyhow::{anyhow, Context};
use candid::Principal as CanisterId;
use candid::{CandidType, Decode, Deserialize, Principal};
use candid_parser::utils::CandidSource;
use clap::Parser;
use dfx_core::canister::build_wallet_canister;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_utils::canister::Argument;
use ic_utils::interfaces::management_canister::builders::{CanisterInstall, CanisterSettings};
use ic_utils::interfaces::management_canister::MgmtMethod;
use ic_utils::interfaces::wallet::{CallForwarder, CallResult};
use ic_utils::interfaces::WalletCanister;
use slog::warn;
use std::option::Option;
use std::path::PathBuf;
use std::str::FromStr;

/// Calls a method on a deployed canister.
#[derive(Parser)]
pub struct CanisterCallOpts {
    /// Specifies the name of the canister to build.
    /// You must specify either a canister name or the --all option.
    canister_name: String,

    /// Specifies the method name to call on the canister.
    method_name: String,

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

    /// Specifies the argument to pass to the method.
    #[arg(conflicts_with("random"), conflicts_with("argument_file"))]
    argument: Option<String>,

    /// Specifies the file from which to read the argument to pass to the method.
    #[arg(
        long,
        value_parser = file_or_stdin_parser,
        conflicts_with("random"),
        conflicts_with("argument")
    )]
    argument_file: Option<PathBuf>,

    /// Specifies the config for generating random argument.
    #[arg(long, conflicts_with("argument"), conflicts_with("argument_file"))]
    random: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[arg(long, requires("argument"), value_parser = ["idl", "raw"])]
    r#type: Option<String>,

    /// Specifies the format for displaying the method's return result.
    #[arg(long, conflicts_with("async"),
        value_parser = ["idl", "raw", "pp"])]
    output: Option<String>,

    /// Specifies the amount of cycles to send on the call.
    /// Deducted from the wallet.
    /// Requires --wallet as a flag to `dfx canister`.
    #[arg(long, value_parser = cycle_amount_parser)]
    with_cycles: Option<u128>,

    /// Provide the .did file with which to decode the response.  Overrides value from dfx.json
    /// for project canisters.
    #[arg(long)]
    candid: Option<PathBuf>,
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
    let (result,): (Result<CallResult, String>,) = builder
        .build()
        .call_and_wait()
        .await
        .context("Failed wallet call.")?;
    Ok(result.map_err(|err| anyhow!(err))?.r#return)
}

async fn request_id_via_wallet_call(
    wallet: &WalletCanister<'_>,
    canister: Principal,
    method_name: &str,
    args: Argument,
    cycles: u128,
) -> DfxResult<ic_agent::RequestId> {
    let call_forwarder: CallForwarder<'_, '_, (CallResult,)> =
        wallet.call(canister, method_name, args, cycles);
    call_forwarder
        .call()
        .await
        .map_err(|err| anyhow!("Agent error {}", err))
}

#[context(
    "Failed to determine effective canister id of method '{}' regarding canister {}.",
    method_name,
    canister_id
)]
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
            MgmtMethod::CreateCanister | MgmtMethod::RawRand
            | MgmtMethod::BitcoinGetBalance | MgmtMethod::BitcoinGetBalanceQuery
            | MgmtMethod::BitcoinGetUtxos | MgmtMethod::BitcoinGetUtxosQuery
            | MgmtMethod::BitcoinSendTransaction | MgmtMethod::BitcoinGetCurrentFeePercentiles
            | MgmtMethod::EcdsaPublicKey | MgmtMethod::SignWithEcdsa => {
                Err(DiagnosedError::new(
                    format!(
                        "{} can only be called by a canister, not by an external user.",
                        method_name.as_ref()
                    ),
                    format!("The easiest way to call {} externally is to proxy this call through a wallet. Try calling this with 'dfx canister call <other arguments> (--network ic) --wallet <wallet id>'.\n\
                    To figure out the id of your wallet, run 'dfx identity get-wallet (--network ic)'.", method_name.as_ref())
                )).context("Method only callable by a canister.")
            }
            MgmtMethod::InstallCode => {
                let install_args = candid::Decode!(arg_value, CanisterInstall)
                    .context("Failed to decode arguments.")?;
                Ok(install_args.canister_id)
            }
            MgmtMethod::UpdateSettings => {
                #[derive(CandidType, Deserialize)]
                struct In {
                    canister_id: CanisterId,
                    settings: CanisterSettings,
                }
                let in_args =
                    candid::Decode!(arg_value, In).context("Failed to decode arguments.")?;
                Ok(in_args.canister_id)
            }
            MgmtMethod::StartCanister
            | MgmtMethod::StopCanister
            | MgmtMethod::CanisterStatus
            | MgmtMethod::DeleteCanister
            | MgmtMethod::DepositCycles
            | MgmtMethod::UninstallCode
            | MgmtMethod::ProvisionalTopUpCanister
            | MgmtMethod::UploadChunk
            | MgmtMethod::ClearChunkStore
            | MgmtMethod::StoredChunks => {
                #[derive(CandidType, Deserialize)]
                struct In {
                    canister_id: CanisterId,
                }
                let in_args =
                    candid::Decode!(arg_value, In).context("Failed to decode arguments.")?;
                Ok(in_args.canister_id)
            }
            MgmtMethod::ProvisionalCreateCanisterWithCycles => {
                Ok(CanisterId::management_canister())
            }
            MgmtMethod::InstallChunkedCode => {
                #[derive(CandidType, Deserialize)]
                struct In {
                    target_canister: Principal,
                }
                let in_args = Decode!(arg_value, In)
                    .context("Argument is not valid for InstallChunkedCode")?;
                Ok(in_args.target_canister)
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
    let agent = env.get_agent();
    fetch_root_key_if_needed(env).await?;

    let callee_canister = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let (canister_id, maybe_local_candid_path) = match CanisterId::from_text(callee_canister) {
        Ok(id) => {
            if let Some(canister_name) = canister_id_store.get_name(callee_canister) {
                get_local_cid_and_candid_path(env, canister_name, Some(id))?
            } else {
                (id, None)
            }
        }
        Err(_) => {
            let canister_id = canister_id_store.get(callee_canister)?;
            get_local_cid_and_candid_path(env, callee_canister, Some(canister_id))?
        }
    };
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

    let is_management_canister = canister_id == CanisterId::management_canister();

    let is_query_method = method_type.as_ref().map(|(_, f)| f.is_query());

    let arguments_from_file = opts
        .argument_file
        .map(|v| arguments_from_file(&v))
        .transpose()?;
    let arguments = opts.argument.as_deref();
    let arguments = arguments_from_file.as_deref().or(arguments);

    let arg_type = opts.r#type.as_deref();
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

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = blob_from_arguments(
        Some(env),
        arguments,
        opts.random.as_deref(),
        arg_type,
        &method_type,
        false,
    )?;

    // amount has been validated by cycle_amount_validator
    let cycles = opts.with_cycles.unwrap_or(0);

    if call_sender == &CallSender::SelectedId && cycles != 0 {
        return Err(DiagnosedError::new("It is only possible to send cycles from a canister.".to_string(), "To send the same function call from your wallet (a canister), run the command using 'dfx canister call <other arguments> (--network ic) --wallet <wallet id>'.\n\
        To figure out the id of your wallet, run 'dfx identity get-wallet (--network ic)'.".to_string())).context("Function caller is not a canister.");
    }

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
                    .with_arg(arg_value)
                    .call()
                    .await
                    .context("Failed query call.")?
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
                    .with_arg(arg_value)
                    .call()
                    .await
                    .context("Failed update call.")?
            }
            CallSender::Wallet(wallet_id) => {
                let wallet = build_wallet_canister(*wallet_id, agent).await?;
                let mut args = Argument::default();
                args.set_raw_arg(arg_value);

                request_id_via_wallet_call(&wallet, canister_id, method_name, args, cycles)
                    .await
                    .context("Failed request via wallet.")?
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
                    .with_arg(arg_value)
                    .call_and_wait()
                    .await
                    .context("Failed update call.")?
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
                .context("Failet to do wallet call.")?
            }
        };

        print_idl_blob(&blob, output_type, &method_type)?;
    }

    Ok(())
}
