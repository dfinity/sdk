use crate::config::dfinity::Config;
use crate::lib::builders::BuildConfig;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::create_canister;
use crate::lib::operations::canister::install_canister;
use ic_agent::{AgentError, InstallMode};
use slog::{info, warn};
use tokio::runtime::Runtime;

pub fn deploy_canisters(env: &dyn Environment, some_canister: Option<&str>) -> DfxResult {
    let log = env.get_logger();

    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mut canister_names = config
        .get_config()
        .get_canister_names_with_dependencies(some_canister)?;
    canister_names.sort();
    let canister_names = canister_names;

    let initial_canister_id_store = CanisterIdStore::for_env(env)?;

    if some_canister.is_some() {
        info!(log, "Deploying: {}", canister_names.join(" "));
    } else {
        info!(log, "Deploying all canisters.");
    }

    register_canisters(env, &canister_names, &initial_canister_id_store)?;

    build_canisters(env, &canister_names, &config)?;

    install_canisters(env, &canister_names, &initial_canister_id_store, &config)?;

    info!(log, "Deployed canisters.");

    Ok(())
}

fn register_canisters(
    env: &dyn Environment,
    canister_names: &[String],
    canister_id_store: &CanisterIdStore,
) -> DfxResult {
    let canisters_to_create = canister_names
        .iter()
        .filter(|n| canister_id_store.find(&n).is_none())
        .cloned()
        .collect::<Vec<String>>();
    if canisters_to_create.is_empty() {
        info!(env.get_logger(), "All canisters have already been created.");
    } else {
        info!(env.get_logger(), "Creating canisters...");
        for canister_name in &canisters_to_create {
            create_canister(env, &canister_name)?;
        }
    }
    Ok(())
}

fn build_canisters(env: &dyn Environment, canister_names: &[String], config: &Config) -> DfxResult {
    info!(env.get_logger(), "Building canisters...");
    let build_mode_check = false;
    let canister_pool = CanisterPool::load(env, build_mode_check, &canister_names)?;

    canister_pool.build_or_fail(BuildConfig::from_config(&config)?)
}

fn install_canisters(
    env: &dyn Environment,
    canister_names: &[String],
    initial_canister_id_store: &CanisterIdStore,
    config: &Config,
) -> DfxResult {
    info!(env.get_logger(), "Installing canisters...");

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    let canister_id_store = CanisterIdStore::for_env(env)?;

    for canister_name in canister_names {
        let (first_mode, second_mode) = match initial_canister_id_store.find(&canister_name) {
            Some(_) => (InstallMode::Upgrade, InstallMode::Install),
            None => (InstallMode::Install, InstallMode::Upgrade),
        };

        let canister_id = canister_id_store.get(&canister_name)?;
        let canister_info = CanisterInfo::load(&config, &canister_name, Some(canister_id))?;
        let compute_allocation = None;
        let result = runtime.block_on(install_canister(
            env,
            &agent,
            &canister_info,
            compute_allocation,
            first_mode,
        ));
        match result {
            Err(DfxError::AgentError(AgentError::ReplicaError {
                reject_code,
                reject_message: _,
            })) if reject_code == 3 || reject_code == 5 => {
                // 3: tried to upgrade a canister that has not been created
                // 5: tried to install a canister that was already installed
                let mode_description = match second_mode {
                    InstallMode::Install => "install",
                    _ => "upgrade",
                };
                warn!(
                    env.get_logger(),
                    "replica error. attempting {}", mode_description
                );
                runtime.block_on(install_canister(
                    env,
                    &agent,
                    &canister_info,
                    compute_allocation,
                    second_mode,
                ))
            }
            other => other,
        }?;
    }

    Ok(())
}
