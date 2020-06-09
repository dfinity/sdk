use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister::CanisterPool;
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
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let logger = env.get_logger();

    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    env.get_cache().install()?;

    let canister_pool = CanisterPool::load(env)?;

    slog::info!(logger, "Building canisters...");

    // TODO: remove the forcing of generating canister id once we have an update flow.
    canister_pool.build_or_fail(
        BuildConfig::from_config(&config)
            .with_generate_id(true)
            .with_skip_frontend(args.is_present("skip-frontend")),
    )?;

    Ok(())
}
