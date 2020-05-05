use crate::commands::canister::create_waiter;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;

use clap::{App, ArgMatches, SubCommand};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("create").about(UserMessage::InstallCanister.to_str())
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches<'_>) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let canister_id = runtime.block_on(agent.create_canister_and_wait(create_waiter()))?;

    eprint!("Canister Id:");
    println!("{:?}", canister_id.to_text());
    Ok(())
}
