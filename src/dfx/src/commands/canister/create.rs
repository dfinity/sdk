use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::progress_bar::ProgressBar;
use crate::lib::waiter::create_waiter;

use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::ManagementCanister;
use std::format;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("create")
        .about(UserMessage::CreateCanister.to_str())
        .arg(
            Arg::with_name("canister_name")
                .takes_value(true)
                .required_unless("all")
                .help(UserMessage::CreateCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .required_unless("canister_name")
                .help(UserMessage::CreateAll.to_str())
                .takes_value(false),
        )
}

fn create_canister(env: &dyn Environment, canister_name: &str) -> DfxResult {
    let message = format!("Creating canister {:?}...", canister_name);
    let b = ProgressBar::new_spinner(&message);

    env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mgr = ManagementCanister::new(
        env.get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?,
    );
    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    let mut canister_id_store = CanisterIdStore::for_env(env)?;

    match canister_id_store.find(&canister_name) {
        Some(canister_id) => {
            let message = format!(
                "{:?} canister was already created and has canister id: {:?}",
                canister_name,
                canister_id.to_text()
            );
            b.finish_with_message(&message);
            Ok(())
        }
        None => {
            let cid = runtime.block_on(mgr.create_canister(create_waiter()))?;
            let canister_id = cid.to_text();
            let message = format!(
                "{:?} canister created with canister id: {:?}",
                canister_name, canister_id
            );
            b.finish_with_message(&message);
            canister_id_store.add(&canister_name, canister_id)
        }
    }?;

    Ok(())
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    if let Some(canister_name) = args.value_of("canister_name") {
        create_canister(env, canister_name)?;
        Ok(())
    } else if args.is_present("all") {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                create_canister(env, canister_name)?;
            }
        }
        Ok(())
    } else {
        Err(DfxError::CanisterNameMissing())
    }
}
