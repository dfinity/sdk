use crate::lib::builders::BuildConfig;
use crate::lib::environment::{AgentEnvironment, Environment};
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
        .arg(
            Arg::with_name("skip-manifest")
                .long("skip-manifest")
                .takes_value(false)
                .help(UserMessage::BuildSkipManifest.to_str()),
        )
        .arg(
            Arg::with_name("provider")
                .help(UserMessage::CanisterComputeProvider.to_str())
                .long("provider")
                .validator(|v| {
                    reqwest::Url::parse(&v)
                        .map(|_| ())
                        .map_err(|_| "should be a valid URL.".to_string())
                })
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    // Need storage for AgentEnvironment ownership.
    let mut _agent_env: Option<AgentEnvironment<'_>> = None;
    let env = if args.is_present("provider") {
        _agent_env = Some(AgentEnvironment::new(
            env,
            args.value_of("provider").expect("Could not find provider."),
        ));
        _agent_env.as_ref().unwrap()
    } else {
        env
    };

    let logger = env.get_logger();

    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    env.get_cache().install()?;

    // First build.
    let canister_pool = CanisterPool::load(env)?;

    // Create canisters on the replica and associate canister ids locally.
    if !args.is_present("skip-manifest") {
        canister_pool.create_canisters(env)?;
    } else {
        slog::warn!(
            env.get_logger(),
            "Skipping the build manifest. Canister IDs might be hard coded."
        );
    }

    slog::info!(logger, "Building canisters...");

    // TODO: remove the forcing of generating canister id once we have an update flow.
    canister_pool.build_or_fail(
        BuildConfig::from_config(&config)
            .with_skip_frontend(args.is_present("skip-frontend"))
            .with_skip_manifest(args.is_present("skip-manifest")),
    )?;

    Ok(())
}
