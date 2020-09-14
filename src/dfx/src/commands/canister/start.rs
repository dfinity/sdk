use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::{Agent, ManagementCanister};
use slog::info;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about(UserMessage::StartCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .required_unless("all")
                .help(UserMessage::StartCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .required_unless("canister_name")
                .help(UserMessage::StartAll.to_str())
                .takes_value(false),
        )
}

async fn start_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_name: &str,
    timeout: Option<&str>,
) -> DfxResult {
    let mgr = ManagementCanister::new(agent);
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id = canister_id_store.get(canister_name)?;

    let duration = expiry_duration(timeout)?;

    info!(
        log,
        "Starting code for canister {}, with canister_id {}",
        canister_name,
        canister_id.to_text(),
    );

    mgr.start_canister(waiter_with_timeout(duration), &canister_id)
        .await
        .map_err(DfxError::from)?;

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

    let timeout = args.value_of("expiry_duration");

    if let Some(canister_name) = args.value_of("canister_name") {
        runtime.block_on(start_canister(env, &agent, &canister_name, timeout))?;
        Ok(())
    } else if args.is_present("all") {
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                runtime.block_on(start_canister(env, &agent, &canister_name, timeout))?;
            }
        }
        Ok(())
    } else {
        Err(DfxError::CanisterNameMissing())
    }
}