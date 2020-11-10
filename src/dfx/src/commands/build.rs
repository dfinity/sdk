use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::create_agent_environment;

use anyhow::anyhow;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

/// Builds all or specific canisters from the code in your project. By default, all canisters are built.
#[derive(Clap)]
#[clap(name("build"))]
pub struct CanisterBuildOpts {
    /// Specifies the name of the canister to build.
    /// You must specify either a canister name or the --all option.
    canister_name: Option<String>,

    /// Builds all canisters configured in the dfx.json file.
    #[clap(long, conflicts_with("canister-name"))]
    all: bool,

    /// Build canisters without creating them. This can be used to check that canisters build ok.
    #[clap(long)]
    check: bool,

    /// Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,
}

pub fn construct() -> App<'static> {
    CanisterBuildOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CanisterBuildOpts = CanisterBuildOpts::from_arg_matches(args);
    let env = create_agent_environment(env, opts.network)?;

    let logger = env.get_logger();

    // Read the config.
    let config = env
        .get_config()
        .ok_or(anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    env.get_cache().install()?;

    let build_mode_check = opts.check;
    let _all = opts.all;

    // Option can be None in which case --all was specified
    let canister_names = config
        .get_config()
        .get_canister_names_with_dependencies(opts.canister_name.as_deref())?;

    // Get pool of canisters to build
    let canister_pool = CanisterPool::load(&env, build_mode_check, &canister_names)?;

    // Create canisters on the replica and associate canister ids locally.
    if build_mode_check {
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
