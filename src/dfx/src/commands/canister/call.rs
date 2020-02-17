use crate::commands::canister::create_waiter;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::{blob_from_arguments, load_idl_file, print_idl_blob};
use clap::{App, Arg, ArgMatches, SubCommand};
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
    let canister_info = CanisterInfo::load(&config, canister_name)?;
    // Read the config.
    let canister_id = canister_info.get_canister_id().ok_or_else(|| {
        DfxError::CannotFindBuildOutputForCanister(canister_info.get_name().to_owned())
    })?;
    let method_name = args
        .value_of("method_name")
        .ok_or_else(|| DfxError::InvalidArgument("method_name".to_string()))?;
    let arguments: Option<&str> = args.value_of("argument");
    let arg_type: Option<&str> = args.value_of("type");

    let idl_ast = load_idl_file(env, canister_info.get_output_idl_path());
    let is_query = if args.is_present("async") {
        false
    } else {
        let is_query_method =
            idl_ast.and_then(|ast| ast.get_method_type(&method_name).map(|f| f.is_query()));
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
    let arg_value = blob_from_arguments(arguments, arg_type)?;
    let client = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    if is_query {
        if let Some(blob) = runtime.block_on(client.query(&canister_id, method_name, &arg_value))? {
            print_idl_blob(&blob)
                .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
        }
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
