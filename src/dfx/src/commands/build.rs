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
            Arg::with_name("canister_name")
                .takes_value(true)
                .conflicts_with("all")
                .help(UserMessage::BuildCanisterName.to_str())
                .required(false),
        )
        .arg(
            Arg::with_name("all")
                .long("all")
                .conflicts_with("canister_name")
                .help(UserMessage::BuildAll.to_str())
                .takes_value(false),
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

    let build_mode_check = args.is_present("check");

    // Option can be None in which case --all was specified
    let some_canister = args.value_of("canister_name");
    let canister_names = config
        .get_config()
        .get_canister_names_with_dependencies(some_canister)?;

    // Get pool of canisters to build
    let canister_pool = CanisterPool::load(&env, build_mode_check, &canister_names)?;

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
            store.get(canister.get_name())?;
        }
    }

    slog::info!(logger, "Building canisters...");

    canister_pool.build_or_fail(
        BuildConfig::from_config(&config)?.with_build_mode_check(build_mode_check),
    )?;

    Ok(())
}
