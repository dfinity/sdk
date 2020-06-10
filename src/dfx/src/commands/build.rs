use crate::config::dfx_version;
use crate::lib::builders::BuildConfig;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
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
            Arg::with_name("skip-regen-id")
                .long("skip-regen-can-id")
                .takes_value(false)
                .help(UserMessage::SkipRegenCID.to_str()),
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

    let canister_pool = CanisterPool::load(env, args.is_present("skip-regen-id"))?;

    //create canisters on the replica and associate canister ids locally
    canister_pool.create_canisters(env)?;

    // First build.
    slog::info!(logger, "Building canisters...");

    // TODO: remove the forcing of generating canister id once we have an update flow.
    canister_pool.build_or_fail(BuildConfig::from_config(&config).with_generate_id(true))?;

    // If there is not a package.json, we don't have a frontend and can quit early.
    if !config.get_project_root().join("package.json").exists() || args.is_present("skip-frontend")
    {
        return Ok(());
    }

    // Frontend build.
    slog::info!(logger, "Building frontend...");
    let mut cmd = std::process::Command::new("npm");
    cmd.arg("run")
        .arg("build")
        .env("DFX_VERSION", &format!("{}", dfx_version()))
        .current_dir(config.get_project_root())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    slog::debug!(logger, "Running {:?}...", cmd);

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(DfxError::BuildError(BuildErrorKind::CompilerError(
            format!("{:?}", cmd),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )));
    } else if !output.stderr.is_empty() {
        // Cannot use eprintln, because it would interfere with the progress bar.
        slog::warn!(logger, "{}", String::from_utf8_lossy(&output.stderr));
    }

    // Second build with assets.
    slog::info!(logger, "Bundling assets with canisters...");
    canister_pool.build_or_fail(BuildConfig::from_config(&config).with_assets(true))?;

    Ok(())
}
