use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
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
    env.get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let canister_name = args.value_of("canister_name").unwrap();
    let canister_id = CanisterIdStore::for_env(env)?.get_canister_id(canister_name)?;

    println!("{}", CanisterId::to_text(&canister_id));
    Ok(())
}
