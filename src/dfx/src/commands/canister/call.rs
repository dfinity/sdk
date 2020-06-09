use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::waiter::create_waiter;
use crate::util::{blob_from_arguments, check_candid_file, print_idl_blob};
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::CanisterId;
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
                .possible_values(&["string", "number", "idl", "raw"]),
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
            let canister_info = CanisterInfo::load(&config, canister_name)?;
            match canister_info.get_canister_id() {
                Some(id) => (id, canister_info.get_output_idl_path()),
                None => return Err(DfxError::InvalidArgument("canister_name".to_string())),
            }
        }
    };

    let method_type = maybe_candid_path.and_then(|path| {
        let (env, actor) = check_candid_file(&path)?;
        let f = actor.get(method_name)?;
        Some((env, f.clone()))
    });
    let is_query_method = match &method_type {
        Some((_, f)) => Some(f.is_query()),
        None => None,
    };

    let arguments: Option<&str> = args.value_of("argument");
    let arg_type: Option<&str> = args.value_of("type");
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
    let arg_value = blob_from_arguments(arguments, arg_type, method_type)?;
    let client = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    if is_query {
        let blob = runtime.block_on(client.query(&canister_id, method_name, &arg_value))?;
        print_idl_blob(&blob)
            .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
    } else if args.is_present("async") {
        let request_id = runtime.block_on(client.call(&canister_id, method_name, &arg_value))?;

        eprint!("Request ID: ");
        println!("0x{}", String::from(request_id));
    } else if let Some(blob) = runtime.block_on(client.call_and_wait(
        &canister_id,
        method_name,
        &arg_value,
        create_waiter(),
    ))? {
        print_idl_blob(&blob)
            .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
    }

    Ok(())
}
