use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_exponential_backoff;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_type, print_idl_blob};

use anyhow::{anyhow, bail, Context};
use clap::Clap;
use ic_types::principal::Principal as CanisterId;
use std::option::Option;
use std::path::PathBuf;

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
    argument: Option<String>,

    /// Specifies the config for generating random argument.
    #[clap(long, conflicts_with("argument"))]
    random: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    r#type: Option<String>,

    /// Specifies the format for displaying the method's return result.
    #[clap(long, conflicts_with("async"),
        possible_values(&["idl", "raw", "pp"]))]
    output: Option<String>,
}

fn get_local_cid_and_candid_path(
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

pub async fn exec(env: &dyn Environment, opts: CanisterCallOpts) -> DfxResult {
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

    let method_type = maybe_candid_path.and_then(|path| get_candid_type(&path, method_name));
    let is_query_method = match &method_type {
        Some((_, f)) => Some(f.is_query()),
        None => None,
    };

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

    if is_query {
        let blob = agent
            .query(&canister_id, method_name)
            .with_arg(&arg_value)
            .call()
            .await?;
        print_idl_blob(&blob, output_type, &method_type)
            .context("Invalid data: Invalid IDL blob.")?;
    } else if opts.r#async {
        let request_id = agent
            .update(&canister_id, method_name)
            .with_arg(&arg_value)
            .call()
            .await?;
        eprint!("Request ID: ");
        println!("0x{}", String::from(request_id));
    } else {
        let blob = agent
            .update(&canister_id, method_name)
            .with_arg(&arg_value)
            .expire_after(timeout)
            .call_and_wait(waiter_with_exponential_backoff())
            .await?;

        print_idl_blob(&blob, output_type, &method_type)
            .context("Invalid data: Invalid IDL blob.")?;
    }

    Ok(())
}
