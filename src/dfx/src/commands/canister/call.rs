use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_type, print_idl_blob};
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use ic_types::principal::Principal as CanisterId;
use std::option::Option;
use tokio::runtime::Runtime;

/// Deletes a canister on the Internet Computer network.
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

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    r#type: Option<String>,

    /// Specifies the format for displaying the method's return result.
    #[clap(long, conflicts_with("async"),
        possible_values(&["idl", "raw", "pp"]))]
    output: Option<String>,
}

pub fn construct() -> App<'static> {
    CanisterCallOpts::into_app().name("call")
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CanisterCallOpts = CanisterCallOpts::from_arg_matches(args);
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let canister_name = opts.canister_name.as_str();
    let method_name = opts.method_name.as_str();

    let (canister_id, maybe_candid_path) = match CanisterId::from_text(canister_name) {
        Ok(id) => {
            // TODO fetch candid file from canister
            (id, None)
        }
        Err(_) => {
            let canister_id = CanisterIdStore::for_env(env)?.get(canister_name)?;

            let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;
            (
                canister_info.get_canister_id()?,
                canister_info.get_output_idl_path(),
            )
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
                    return Err(DfxError::InvalidMethodCall(format!(
                        "{} is not a query method",
                        method_name
                    )));
                } else {
                    false
                }
            }
            None => opts.query,
        }
    };

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = blob_from_arguments(arguments, arg_type, &method_type)?;
    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    let timeout = expiry_duration();

    if is_query {
        let blob = runtime.block_on(
            agent
                .query(&canister_id, method_name)
                .with_arg(&arg_value)
                .call(),
        )?;
        print_idl_blob(&blob, output_type, &method_type)
            .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
    } else if args.is_present("async") {
        let request_id = runtime.block_on(
            agent
                .update(&canister_id, &method_name)
                .with_arg(&arg_value)
                .call(),
        )?;
        eprint!("Request ID: ");
        println!("0x{}", String::from(request_id));
    } else {
        let blob = runtime.block_on(
            agent
                .update(&canister_id, &method_name)
                .with_arg(&arg_value)
                .expire_after(timeout)
                .call_and_wait(waiter_with_timeout(timeout)),
        )?;

        print_idl_blob(&blob, output_type, &method_type)
            .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
    }

    Ok(())
}
