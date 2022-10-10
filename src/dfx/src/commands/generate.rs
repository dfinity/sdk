use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::create_agent_environment;
use crate::NetworkOpt;

use clap::Parser;
use tokio::runtime::Runtime;

/// Generate type declarations for canisters from the code in your project
#[derive(Parser)]
pub struct GenerateOpts {
    /// Specifies the name of the canister to generate type information for.
    /// If you do not specify a canister name, generates types for all canisters.
    canister_name: Option<String>,

    #[clap(flatten)]
    network: NetworkOpt,
}

pub fn exec(env: &dyn Environment, opts: GenerateOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network.network)?;

    // Read the config.
    let config = env.get_config_or_anyhow()?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    env.get_cache().install()?;

    // Option can be None which means generate types for all canisters
    let canister_names = config
        .get_config()
        .get_canister_names_with_dependencies(opts.canister_name.as_deref())?;

    // Get pool of canisters to build
    let canister_pool = CanisterPool::load(&env, false, &canister_names)?;

    // This is just to display an error if trying to generate before creating the canister.
    let store = CanisterIdStore::for_env(&env)?;
    // If generate for motoko canister, build first
    let mut build_before_generate = false;
    for canister in canister_pool.get_canister_list() {
        let canister_name = canister.get_name();
        let canister_id = store.get(canister_name)?;
        if let Some(info) = canister_pool.get_canister_info(&canister_id) {
            if info.is_motoko() {
                build_before_generate = true;
            }
        }
    }

    let build_config = BuildConfig::from_config(&config)?;

    if build_before_generate {
        slog::info!(
            env.get_logger(),
            "Building canisters before generate for Motoko"
        );
        let runtime = Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(canister_pool.build_or_fail(&build_config))?;
    }

    for canister in canister_pool.get_canister_list() {
        canister.generate(&canister_pool, &build_config)?;
    }

    Ok(())
}
