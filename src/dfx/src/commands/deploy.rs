use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use crate::lib::operations::canister::deploy_canisters;
use crate::lib::provider::create_agent_environment;
use crate::util::expiry_duration;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static> {
    SubCommand::with_name("deploy")
        .about(UserMessage::DeployCanister.to_str())
        .arg(
            Arg::new("canister_name")
                .takes_value(true)
                //.help(UserMessage::DeployCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::new("network")
                //.help(UserMessage::CanisterComputeNetwork.to_str())
                .long("network")
                .takes_value(true),
        )
        .arg(
            Arg::new("argument")
                //.help(UserMessage::ArgumentValue.to_str())
                .long("argument")
                .takes_value(true),
        )
        .arg(
            Arg::new("type")
                //.help(UserMessage::ArgumentType.to_str())
                .long("type")
                .takes_value(true)
                .requires("argument")
                .possible_values(&["idl", "raw"]),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let env = create_agent_environment(env, args)?;

    let timeout = expiry_duration();
    let canister = args.value_of("canister_name");

    let argument = args.value_of("argument");
    let argument_type = args.value_of("type");

    deploy_canisters(&env, canister, argument, argument_type, timeout)
}
