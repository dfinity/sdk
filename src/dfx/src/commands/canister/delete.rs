use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::create_waiter;

use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::{Agent, ManagementCanister};
use slog::info;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("delete")
        .about(UserMessage::DeleteCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .required_unless("all")
                .help(UserMessage::DeleteCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .required_unless("canister_name")
                .help(UserMessage::DeleteAll.to_str())
                .takes_value(false),
        )
}

async fn delete_canister(env: &dyn Environment, agent: &Agent, canister_name: &str) -> DfxResult {
    let mgr = ManagementCanister::new(agent);
    let log = env.get_logger();
    let mut canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id = canister_id_store.get(canister_name)?;
    info!(
        log,
        "Deleting code for canister {}, with canister_id {}",
        canister_name,
        canister_id.to_text(),
    );

    mgr.delete_canister(create_waiter(), &canister_id)
        .await
        .map_err(DfxError::from)?;

    canister_id_store.remove(canister_name)?;

    Ok(())
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    if let Some(canister_name) = args.value_of("canister_name") {
        runtime.block_on(delete_canister(env, &agent, &canister_name))?;
        Ok(())
    } else if args.is_present("all") {
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                runtime.block_on(delete_canister(env, &agent, &canister_name))?;
            }
        }
        Ok(())
    } else {
        Err(DfxError::CanisterNameMissing())
    }
}
