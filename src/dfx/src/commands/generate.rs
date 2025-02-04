use crate::config::cache::VersionCache;
use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::network::network_opt::NetworkOpt;
use clap::Parser;
use tokio::runtime::Runtime;

/// Generate type declarations for canisters from the code in your project
#[derive(Parser)]
pub struct GenerateOpts {
    /// Specifies the name of the canister to generate type information for.
    /// If you do not specify a canister name, generates types for all canisters.
    canister_name: Option<String>,

    #[command(flatten)]
    network: NetworkOpt,
}

pub fn exec(env: &dyn Environment, opts: GenerateOpts) -> DfxResult {
    let env = create_anonymous_agent_environment(env, opts.network.to_network_name())?;
    let log = env.get_logger();

    // Read the config.
    let config = env.get_config_or_anyhow()?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    VersionCache::install(&env, &env.get_cache().version_str())?;

    // Option can be None which means generate types for all canisters
    let canisters_to_load = config
        .get_config()
        .get_canister_names_with_dependencies(opts.canister_name.as_deref())?;
    let canisters_to_generate = canisters_to_load.clone().into_iter().collect();

    let canister_pool_load = CanisterPool::load(&env, false, &canisters_to_load)?;

    // If generate for motoko canister, build first
    let mut build_before_generate = Vec::new();
    let mut build_dependees = Vec::new();
    for canister in canister_pool_load.get_canister_list() {
        let canister_name = canister.get_name();
        if let Some(info) = canister_pool_load.get_first_canister_with_name(canister_name) {
            if info.get_info().is_motoko() {
                build_before_generate.push(canister_name.to_string());
            }
            for dependent_canister in config
                .get_config()
                .get_canister_names_with_dependencies(Some(canister_name))?
            {
                if !build_dependees.contains(&dependent_canister) {
                    build_dependees.push(dependent_canister);
                }
            }
        }
    }
    let build_config =
        BuildConfig::from_config(&config)?.with_canisters_to_build(build_before_generate);
    let generate_config =
        BuildConfig::from_config(&config)?.with_canisters_to_build(canisters_to_generate);

    if build_config
        .canisters_to_build
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        let canister_pool_build = CanisterPool::load(&env, true, &build_dependees)?;
        slog::info!(log, "Building canisters before generate for Motoko");
        let runtime = Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(canister_pool_build.build_or_fail(&env, log, &build_config))?;
    }

    for canister in canister_pool_load.canisters_to_build(&generate_config) {
        canister.generate(&env, log, &canister_pool_load, &generate_config)?;
    }

    Ok(())
}
