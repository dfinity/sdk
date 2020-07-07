use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::create_agent_environment;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about(UserMessage::BuildCanister.to_str())
        .arg(
            Arg::with_name("skip-frontend")
                .long("skip-frontend")
                .takes_value(false)
                .help(UserMessage::SkipFrontend.to_str()),
        )
        .arg(
            Arg::with_name("check")
                .long("check")
                .takes_value(false)
                .help(UserMessage::BuildCheck.to_str()),
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

    let logger = env.get_logger();

    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    env.get_cache().install()?;

    // First build.
    let canister_pool = CanisterPool::load(&env)?;

    // Create canisters on the replica and associate canister ids locally.
    if args.is_present("check") {
        slog::warn!(
            env.get_logger(),
            "Building canisters to check they build ok. Canister IDs might be hard coded."
        );
    } else {
        // CanisterIds would have been set in CanisterPool::load, if available.
        // This is just to display an error if trying to build before creating the canister.
        let store = CanisterIdStore::for_env(&env)?;
        for canister in canister_pool.get_canister_list() {
            store.get_canister_id(canister.get_name())?;
        }
    }

    slog::info!(logger, "Building canisters...");

    // TODO: remove the forcing of generating canister id once we have an update flow.
    canister_pool.build_or_fail(
        BuildConfig::from_config(&config)
            .with_skip_frontend(args.is_present("skip-frontend"))
            .with_build_mode_check(args.is_present("check")),
    )?;

    Ok(())
}
