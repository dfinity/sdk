use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::CanisterId;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("id")
        .about(UserMessage::IdCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .help(UserMessage::CanisterName.to_str())
                .required(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let canister_name = args.value_of("canister_name").unwrap();
    let canister_info = CanisterInfo::load(&config, canister_name)?;
    let canister_id = canister_info.get_canister_id().map_err(|_| {
        DfxError::CannotFindBuildOutputForCanister(canister_info.get_name().to_owned())
    })?;
    println!("{}", CanisterId::to_text(&canister_id));
    Ok(())
}
