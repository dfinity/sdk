use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::get_local_cid_and_candid_path;
use crate::lib::sign::sign_transport::SignReplicaV1Transport;
use crate::lib::sign::signed_message::SignedMessageV1;

use crate::util::{blob_from_arguments, get_candid_type};

use ic_agent::AgentError;
use ic_types::principal::Principal;

use anyhow::{anyhow, bail};
use chrono::Utc;
use clap::Clap;
use slog::info;
use std::option::Option;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Sign a canister call and generate message file in json
#[derive(Clap)]
pub struct CanisterSignOpts {
    /// Specifies the name of the canister to call.
    canister_name: String,

    /// Specifies the method name to call on the canister.
    method_name: String,

    /// Sends a query request to a canister.
    #[clap(long)]
    query: bool,

    /// Sends an update request to a canister. This is the default if the method is not a query method.
    #[clap(long, conflicts_with("query"))]
    update: bool,

    /// Specifies the argument to pass to the method.
    argument: Option<String>,

    /// Specifies the config for generating random argument.
    #[clap(long, conflicts_with("argument"))]
    random: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    r#type: Option<String>,

    /// Specifies how long will the message be valid in seconds, default to be 300s (5 minutes)
    #[clap(long, default_value("300"))]
    expire_after: u64,

    /// Specifies the output file name.
    #[clap(long, default_value("message.json"))]
    file: String,
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterSignOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();
    if *call_sender != CallSender::SelectedId {
        bail!("`sign` currently doesn't support proxy through wallet canister, please use `dfx canister --no-wallet sign ...`.");
    }

    let callee_canister = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();
    let canister_id_store = CanisterIdStore::for_env(env)?;

    let (canister_id, maybe_candid_path) = match Principal::from_text(callee_canister) {
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

    let method_type = maybe_candid_path.and_then(|path| get_candid_type(&path, method_name));
    let is_query_method = match &method_type {
        Some((_, f)) => Some(f.is_query()),
        None => None,
    };

    let arguments = opts.argument.as_deref();
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
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    let network = env
        .get_network_descriptor()
        .expect("Cannot get network descriptor from environment.")
        .providers
        .first()
        .expect("Cannot get network provider (url).")
        .to_string();

    let sender = env
        .get_selected_identity_principal()
        .expect("Selected identity not instantiated.");

    let timeout = Duration::from_secs(opts.expire_after);
    let expiration_system_time = SystemTime::now()
        .checked_add(timeout)
        .ok_or_else(|| anyhow!("Time wrapped around."))?;
    let chorono_timeout = chrono::Duration::seconds(opts.expire_after as i64);
    let creation = Utc::now();
    let expiration = creation
        .checked_add_signed(chorono_timeout)
        .ok_or_else(|| anyhow!("Expiration datetime overflow."))?;

    let message_template = SignedMessageV1::new(
        creation,
        expiration,
        network,
        sender,
        canister_id.clone(),
        method_name.to_string(),
        arg_value.clone(),
    );

    let file_name = opts.file;
    if Path::new(&file_name).exists() {
        bail!(
            "[{}] already exists, please specify a different output file name.",
            file_name
        );
    }

    let mut sign_agent = agent.clone();
    sign_agent.set_transport(SignReplicaV1Transport::new(file_name, message_template));

    if is_query {
        let res = sign_agent
            .query(&canister_id, method_name)
            .with_arg(&arg_value)
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
            .with_arg(&arg_value)
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
    }
}
