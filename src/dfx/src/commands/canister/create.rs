use crate::commands::canister::create_waiter;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;

use clap::{App, Arg, ArgMatches, SubCommand};

use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("create")
    	.about(UserMessage::InstallCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .required_unless("all")
                // .help(UserMessage::InstallCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .required_unless("canister_name")
                // .help(UserMessage::InstallAll.to_str())
                .takes_value(false),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let log = env.get_logger();
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    if let Some(canister_name) = args.value_of("canister_name") {
        let canister_info = CanisterInfo::load(&config, canister_name)?;

	    runtime.block_on(agent.create_canister_and_wait(create_waiter()))
	        .map(|_| ())
	        .map_err(DfxError::from)
        
    } else if args.is_present("all") {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                let canister_info = CanisterInfo::load(&config, canister_name)?;
                    runtime.block_on(agent.create_canister_and_wait(create_waiter()))?;
            }
        }
        Ok(())
    } else {
        Err(DfxError::CanisterNameMissing())
    }
}