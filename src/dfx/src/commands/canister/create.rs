use crate::commands::canister::create_waiter;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister::{CanManMetadata, CanisterManifest};

use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::CanisterId;
use serde_json::Map;
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

fn create_canister(env: &dyn Environment, canister_name: &str) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let info = CanisterInfo::load(&config, canister_name)?;

    let manifest_path = info.get_manifest_path();
    // check if the canister_manifest.json file exists

    if manifest_path.is_file() {
        {
            let file = std::fs::File::open(info.get_manifest_path()).unwrap();
            let mut manifest: CanisterManifest = serde_json::from_reader(file).unwrap();

            match manifest.canisters.get(info.get_name().clone()) {
                Some(serde_value) => {
                    let metadata: CanManMetadata =
                        serde_json::from_value(serde_value.to_owned()).unwrap();
                    CanisterId::from_text(metadata.canister_id).ok();
                    ()
                }
                None => {
                    let cid = runtime.block_on(agent.create_canister_and_wait(create_waiter()))?;
                    info.set_canister_id(cid.clone())?;
                    manifest.add_entry(&info, cid.clone())?;
                    ()
                }
            }
        }
    } else {
        let cid = runtime.block_on(agent.create_canister_and_wait(create_waiter()))?;
        info.set_canister_id(cid.clone())?;
        let mut manifest = CanisterManifest {
            canisters: Map::new(),
        };
        manifest.add_entry(&info, cid.clone())?;
    }
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
