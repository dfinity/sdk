use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::{blob_from_arguments, print_idl_blob};
use clap::{App, Arg, ArgMatches};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static> {
    App::new("query")
        .about(UserMessage::QueryCanister.to_str())
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
            Arg::with_name("argument")
                .help(UserMessage::ArgumentValue.to_str())
                .takes_value(true),
        )
        .arg(
            Arg::with_name("type")
                .help(UserMessage::ArgumentType.to_str())
                .long("type")
                .takes_value(true)
                .requires("argument")
                .possible_values(&["string", "number", "idl"]),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let canister_name = args.value_of("canister_name").unwrap();
    let canister_info = CanisterInfo::load(&config, canister_name)?;
    let canister_id = canister_info.get_canister_id().ok_or_else(|| {
        DfxError::CannotFindBuildOutputForCanister(canister_info.get_name().to_owned())
    })?;

    let method_name = args.value_of("method_name").unwrap();
    let arguments: Option<&str> = args.value_of("argument");
    let arg_type: Option<&str> = args.value_of("type");

    let arg_value = blob_from_arguments(arguments, arg_type)?;
    eprintln!(
        r#"The 'canister query' command has been deprecated. Please use the 'canister call' command."#
    );

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let blob = runtime.block_on(agent.query(&canister_id, method_name, &arg_value))?;
    print_idl_blob(&blob).map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;

    Ok(())
}
