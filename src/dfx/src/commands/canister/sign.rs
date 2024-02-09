use crate::commands::canister::call::get_effective_canister_id;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::get_local_cid_and_candid_path;
use crate::lib::sign::sign_transport::SignTransport;
use crate::lib::sign::signed_message::SignedMessageV1;
use crate::util::clap::parsers::file_or_stdin_parser;
use crate::util::{arguments_from_file, blob_from_arguments, get_candid_type};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use candid_parser::utils::CandidSource;
use clap::Parser;
use dfx_core::identity::CallSender;
use ic_agent::AgentError;
use ic_agent::RequestId;
use slog::info;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use time::OffsetDateTime;

/// Sign a canister call and generate message file.
#[derive(Parser)]
pub struct CanisterSignOpts {
    /// Specifies the name of the canister to call.
    canister_name: String,

    /// Specifies the method name to call on the canister.
    method_name: String,

    /// Sends a query request to a canister.
    #[arg(long)]
    query: bool,

    /// Sends an update request to a canister. This is the default if the method is not a query method.
    #[arg(long, conflicts_with("query"))]
    update: bool,

    /// Specifies the argument to pass to the method.
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
    #[arg(long, conflicts_with("argument"))]
    random: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[arg(long, requires("argument"), value_parser = ["idl", "raw"])]
    r#type: Option<String>,

    /// Specifies how long the message will be valid in seconds, default to be 300s (5 minutes)
    #[arg(long, default_value = "5m")]
    expire_after: String,

    /// Specifies the output file name.
    #[arg(long, default_value = "message.json")]
    file: PathBuf,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterSignOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();
    if *call_sender != CallSender::SelectedId {
        bail!("`sign` currently doesn't support proxying through the wallet canister, please use `dfx canister sign --no-wallet ...`.");
    }

    let callee_canister = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let (canister_id, maybe_candid_path) = match Principal::from_text(callee_canister) {
        Ok(id) => {
            if let Some(canister_name) = canister_id_store.get_name(callee_canister) {
                get_local_cid_and_candid_path(env, canister_name, Some(id))?
            } else {
                // Sign works in offline mode, cannot fetch from remote canister
                (id, None)
            }
        }
        Err(_) => {
            let canister_id = canister_id_store.get(callee_canister)?;
            get_local_cid_and_candid_path(env, callee_canister, Some(canister_id))?
        }
    };

    let method_type =
        maybe_candid_path.and_then(|path| get_candid_type(CandidSource::File(&path), method_name));
    let is_query_method = method_type.as_ref().map(|(_, f)| f.is_query());

    let arguments_from_file = opts
        .argument_file
        .map(|v| arguments_from_file(&v))
        .transpose()?;
    let arguments = opts.argument.as_deref();
    let arguments = arguments_from_file.as_deref().or(arguments);

    let arg_type = opts.r#type.as_deref();
    let is_query = match is_query_method {
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
    };

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = blob_from_arguments(arguments, opts.random.as_deref(), arg_type, &method_type)?;
    let agent = env.get_agent();

    let network = env
        .get_network_descriptor()
        .providers
        .first()
        .expect("Cannot get network provider (url).")
        .to_string();

    let sender = env
        .get_selected_identity_principal()
        .expect("Selected identity not instantiated.");

    let timeout = humantime::parse_duration(&opts.expire_after)
        .map_err(|_| anyhow!("Cannot parse expire_after as a duration (e.g. `1h`, `1h 30m`)"))?;
    //let timeout = Duration::from_secs(opts.expire_after);
    let expiration_system_time = SystemTime::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow!("Time wrapped around."))?;
    let creation = OffsetDateTime::now_utc();
    let expiration = creation
        .checked_add(timeout.try_into()?)
        .ok_or_else(|| anyhow!("Expiration datetime overflow."))?;

    let message_template = SignedMessageV1::new(
        creation,
        expiration,
        network,
        sender,
        canister_id,
        method_name.to_string(),
        arg_value.clone(),
    );

    let file_name = opts.file;
    if Path::new(&file_name).exists() {
        bail!(
            "[{}] already exists, please specify a different output file name.",
            file_name.display(),
        );
    }

    let mut sign_agent = agent.clone();
    sign_agent.set_transport(SignTransport::new(file_name.clone(), message_template));

    let is_management_canister = canister_id == Principal::management_canister();
    let effective_canister_id =
        get_effective_canister_id(is_management_canister, method_name, &arg_value, canister_id)?;

    if is_query {
        let res = sign_agent
            .query(&canister_id, method_name)
            .with_effective_canister_id(effective_canister_id)
            .with_arg(arg_value)
            .expire_at(expiration_system_time)
            .call()
            .await;
        match res {
            Err(AgentError::TransportError(b)) => {
                info!(log, "{}", b);
                Ok(())
            }
            Err(e) => bail!(e),
            Ok(_) => unreachable!(),
        }
    } else {
        let res = sign_agent
            .update(&canister_id, method_name)
            .with_effective_canister_id(effective_canister_id)
            .with_arg(arg_value)
            .expire_at(expiration_system_time)
            .call()
            .await;
        match res {
            Err(AgentError::TransportError(b)) => {
                info!(log, "{}", b);
                //Ok(())
            }
            Err(e) => bail!(e),
            Ok(_) => unreachable!(),
        }
        let path = Path::new(&file_name);
        let mut file = File::open(path).map_err(|_| anyhow!("Message file doesn't exist."))?;
        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|_| anyhow!("Cannot read the message file."))?;
        let message: SignedMessageV1 =
            serde_json::from_str(&json).map_err(|_| anyhow!("Invalid json message."))?;
        // message from file guaranteed to have request_id becase it is a update message just generated
        let request_id = RequestId::from_str(&message.request_id.unwrap())
            .context("Failed to parse request id.")?;
        let res = sign_agent
            .request_status_raw(&request_id, canister_id)
            .await;
        match res {
            Err(AgentError::TransportError(b)) => {
                info!(log, "{}", b);
                Ok(())
            }
            Err(e) => bail!(e),
            Ok(_) => unreachable!(),
        }
    }
}
