use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_type, print_idl_blob};
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_types::principal::Principal as CanisterId;
use std::option::Option;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("call")
        .about(UserMessage::CallCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .help(UserMessage::CanisterName.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name("method_name")
                .help(UserMessage::MethodName.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name("async")
                .help(UserMessage::AsyncResult.to_str())
                .long("async")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("query")
                .help(UserMessage::QueryCanister.to_str())
                .long("query")
                .conflicts_with("async")
                .conflicts_with("update")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("update")
                .help(UserMessage::UpdateCanisterArg.to_str())
                .long("update")
                .conflicts_with("async")
                .conflicts_with("query")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("type")
                .help(UserMessage::ArgumentType.to_str())
                .long("type")
                .takes_value(true)
                .requires("argument")
                .possible_values(&["idl", "raw"]),
        )
        .arg(
            Arg::with_name("output")
                .help(UserMessage::OutputType.to_str())
                .long("output")
                .takes_value(true)
                .conflicts_with("async")
                .possible_values(&["idl", "raw", "pp"]),
        )
        .arg(
            Arg::with_name("argument")
                .help(UserMessage::ArgumentValue.to_str())
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let canister_name = args.value_of("canister_name").unwrap();
    let method_name = args
        .value_of("method_name")
        .ok_or_else(|| DfxError::InvalidArgument("method_name".to_string()))?;

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

    let arguments: Option<&str> = args.value_of("argument");
    let arg_type: Option<&str> = args.value_of("type");
    let output_type: Option<&str> = args.value_of("output");
    let is_query = if args.is_present("async") {
        false
    } else {
        match is_query_method {
            Some(true) => !args.is_present("update"),
            Some(false) => {
                if args.is_present("query") {
                    return Err(DfxError::InvalidMethodCall(format!(
                        "{} is not a query method",
                        method_name
                    )));
                } else {
                    false
                }
            }
            None => args.is_present("query"),
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
