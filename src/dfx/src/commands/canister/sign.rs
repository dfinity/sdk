use crate::commands::canister::call::get_effective_canister_id;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister::get_canister_id_and_candid_path;
use crate::lib::sign::signed_message::SignedMessageV1;
use crate::util::clap::argument_from_cli::ArgumentFromCliPositionalOpt;
use crate::util::{blob_from_arguments, get_candid_type};
use anyhow::{anyhow, bail};
use candid::Principal;
use candid_parser::utils::CandidSource;
use clap::Parser;
use dfx_core::identity::CallSender;
use dfx_core::json::save_json_file;
use ic_utils::interfaces::management_canister::MgmtMethod;
use slog::info;
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::SystemTime;
use time::OffsetDateTime;

/// Sign a canister call and generate message file.
#[derive(Parser)]
pub struct CanisterSignOpts {
    /// Specifies the name/id of the canister to call.
    canister_name: String,

    /// Specifies the method name to call on the canister.
    method_name: String,

    #[command(flatten)]
    argument_from_cli: ArgumentFromCliPositionalOpt,

    /// Sends a query request to a canister.
    #[arg(long)]
    query: bool,

    /// Sends an update request to a canister. This is the default if the method is not a query method.
    #[arg(long, conflicts_with("query"))]
    update: bool,

    /// Specifies the config for generating random argument.
    #[arg(
        long,
        conflicts_with("argument"),
        conflicts_with("argument_file"),
        conflicts_with("always_assist")
    )]
    random: Option<String>,

    /// Specifies how long the message will be valid in seconds, default to be 300s (5 minutes)
    #[arg(long, default_value = "5m")]
    expire_after: String,

    /// Specifies the output file name.
    #[arg(long, default_value = "message.json")]
    file: PathBuf,

    /// Always use Candid assist when the argument types are all optional.
    #[arg(
        long,
        conflicts_with("argument"),
        conflicts_with("argument_file"),
        conflicts_with("random")
    )]
    always_assist: bool,
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

    let method_name = opts.method_name.as_str();

    let (canister_id, maybe_candid_path) =
        get_canister_id_and_candid_path(env, opts.canister_name.as_str())?;

    let method_type =
        maybe_candid_path.and_then(|path| get_candid_type(CandidSource::File(&path), method_name));
    let is_query_method = method_type.as_ref().map(|(_, f)| f.is_query());

    let (argument_from_cli, argument_type) = opts.argument_from_cli.get_argument_and_type()?;
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
    let arg_value = blob_from_arguments(
        Some(env),
        argument_from_cli.as_deref(),
        opts.random.as_deref(),
        argument_type.as_deref(),
        &method_type,
        false,
        opts.always_assist,
    )?;
    let agent = env.get_agent();

    let network_descriptor = env.get_network_descriptor();
    let network = network_descriptor
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
        network_descriptor.is_ic,
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

    let effective_canister_id = if canister_id == Principal::management_canister() {
        let management_method = MgmtMethod::from_str(method_name).map_err(|_| {
            anyhow!(
                "Attempted to call an unsupported management canister method: {}",
                method_name
            )
        })?;
        get_effective_canister_id(&management_method, &arg_value)?
    } else {
        canister_id
    };

    if is_query {
        let signed_query = agent
            .query(&canister_id, method_name)
            .with_effective_canister_id(effective_canister_id)
            .with_arg(arg_value)
            .expire_at(expiration_system_time)
            .sign()?;
        let message = message_template
            .clone()
            .with_call_type("query".to_string())
            .with_content(hex::encode(signed_query.signed_query));
        save_json_file(&file_name, &message)?;
        info!(log, "Query message generated at [{}]", file_name.display());
        Ok(())
    } else {
        let signed_update = agent
            .update(&canister_id, method_name)
            .with_effective_canister_id(effective_canister_id)
            .with_arg(arg_value)
            .expire_at(expiration_system_time)
            .sign()?;
        let request_id = signed_update.request_id;
        let message = message_template
            .clone()
            .with_call_type("update".to_string())
            .with_request_id(request_id)
            .with_content(hex::encode(&signed_update.signed_update));
        let signed_request_status = agent.sign_request_status(canister_id, request_id)?;
        let message = message
            .with_signed_request_status(hex::encode(signed_request_status.signed_request_status));
        save_json_file(&file_name, &message)?;
        info!(
            log,
            "Update and request_status message generated at [{}]",
            file_name.display()
        );
        Ok(())
    }
}
