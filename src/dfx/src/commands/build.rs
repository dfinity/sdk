use crate::config::cache::DiskBasedCache;
use crate::lib::agent::create_agent_environment;
use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::network::network_opt::NetworkOpt;
use crate::lib::operations::canister::add_canisters_with_ids;
use clap::Parser;
use std::path::PathBuf;
use tokio::runtime::Runtime;

/// Builds all or specific canisters from the code in your project. By default, all canisters are built.
#[derive(Parser)]
pub struct CanisterBuildOpts {
    /// Specifies the name of the canister to build.
    /// You must specify either a canister name or the --all option.
    canister_name: Option<String>,

    /// Builds all canisters configured in the dfx.json file.
    #[arg(long, conflicts_with("canister_name"))]
    all: bool,

    /// Build canisters without creating them. This can be used to check that canisters build ok.
    #[arg(long)]
    check: bool,

    /// Output environment variables to a file in dotenv format (without overwriting any user-defined variables, if the file already exists).
    #[arg(long)]
    output_env_file: Option<PathBuf>,

    #[command(flatten)]
    network: NetworkOpt,
}

pub fn exec(env: &dyn Environment, opts: CanisterBuildOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network.to_network_name())?;

    let logger = env.get_logger();

    // Read the config.
    let config = env.get_config_or_anyhow()?;
    let env_file = config.get_output_env_file(opts.output_env_file)?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    DiskBasedCache::install(&env.get_cache().version_str())?;

    let build_mode_check = opts.check;

    // Option can be None in which case --all was specified
    let required_canisters = config
        .get_config()
        .get_canister_names_with_dependencies(opts.canister_name.as_deref())?;
    let canisters_to_load = add_canisters_with_ids(&required_canisters, &env, &config);

    let canisters_to_build = required_canisters
        .into_iter()
        .filter(|canister_name| {
            !config
                .get_config()
                .is_remote_canister(canister_name, &env.get_network_descriptor().name)
                .unwrap_or(false)
        })
        .collect();

    let canister_pool = CanisterPool::load(&env, build_mode_check, &canisters_to_load)?;

    // Create canisters on the replica and associate canister ids locally.
    if build_mode_check {
        slog::warn!(
            logger,
            "Building canisters to check they build ok. Canister IDs might be hard coded."
        );
    } else {
        // CanisterIds would have been set in CanisterPool::load, if available.
        // This is just to display an error if trying to build before creating the canister.
        let store = env.get_canister_id_store()?;
        for canister in canister_pool.get_canister_list() {
            let canister_name = canister.get_name();
            store.get(canister_name)?;
        }
    }

    slog::info!(logger, "Building canisters...");

    let runtime = Runtime::new().expect("Unable to create a runtime");
    let build_config =
        BuildConfig::from_config(&config, env.get_network_descriptor().is_playground())?
            .with_build_mode_check(build_mode_check)
            .with_canisters_to_build(canisters_to_build)
            .with_env_file(env_file);
    runtime.block_on(canister_pool.build_or_fail(logger, &build_config))?;

    Ok(())
}
