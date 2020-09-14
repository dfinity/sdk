use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use crate::lib::operations::canister::deploy_canisters;
use crate::lib::provider::create_agent_environment;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("deploy")
        .about(UserMessage::DeployCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .help(UserMessage::DeployCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::with_name("network")
                .help(UserMessage::CanisterComputeNetwork.to_str())
                .long("network")
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let env = create_agent_environment(env, args)?;

    let timeout = args.value_of("expiry_duration");
    let canister = args.value_of("canister_name");

    deploy_canisters(&env, canister, timeout)
}
