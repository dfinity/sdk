use crate::config::dfx_version;
use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister::CanisterPool;
use clap::{App, Arg, ArgMatches};

pub fn construct() -> App<'static> {
    App::new("build")
        .about(UserMessage::BuildCanister.to_str())
        .arg(
            Arg::with_name("skip-frontend")
                .long("skip-frontend")
                .takes_value(false)
                .help(UserMessage::SkipFrontend.to_str()),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let logger = env.get_logger();

    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    env.get_cache().install()?;

    let canister_pool = CanisterPool::load(env)?;
    // First build.
    slog::info!(logger, "Building canisters...");
    canister_pool.build_or_fail(BuildConfig::from_config(config.get_config()))?;

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
    canister_pool.build_or_fail(BuildConfig::from_config(config.get_config()).with_assets(true))?;

    Ok(())
}
