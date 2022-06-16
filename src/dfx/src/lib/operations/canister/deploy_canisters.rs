use crate::config::dfinity::Config;
use crate::lib::builders::BuildConfig;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister::CanisterPool;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::{create_canister, install_canister};
use crate::util::{blob_from_arguments, get_candid_init_type};

use anyhow::{anyhow, bail};
use fn_error_context::context;
use humanize_rs::bytes::Bytes;
use ic_agent::AgentError;
use ic_utils::interfaces::management_canister::attributes::{
    ComputeAllocation, FreezingThreshold, MemoryAllocation,
};
use ic_utils::interfaces::management_canister::builders::InstallMode;
use slog::info;
use std::convert::TryFrom;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
#[context("Failed while trying to deploy canisters.")]
pub async fn deploy_canisters(
    env: &dyn Environment,
    some_canister: Option<&str>,
    argument: Option<&str>,
    argument_type: Option<&str>,
    force_reinstall: bool,
    upgrade_unchanged: bool,
    timeout: Duration,
    with_cycles: Option<&str>,
    call_sender: &CallSender,
    create_call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();

    let config = env
        .get_config()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;
    let initial_canister_id_store = CanisterIdStore::for_env(env)?;

    let network = env.get_network_descriptor();

    let canisters_to_build = canister_with_dependencies(&config, some_canister)?;

    let canisters_to_deploy = if force_reinstall {
        // don't force-reinstall the dependencies too.
        match some_canister {
            Some(canister_name) => {
                if config.get_config().is_remote_canister(canister_name, &network.name)? {
                    bail!("The '{}' canister is remote for network '{}' and cannot be force-reinstalled from here",
                    canister_name, &network.name);
                }
                vec!(String::from(canister_name))
            },
            None => bail!("The --mode=reinstall is only valid when deploying a single canister, because reinstallation destroys all data in the canister."),
        }
    } else {
        canisters_to_build.clone()
    };
    let canisters_to_deploy: Vec<String> = canisters_to_deploy
        .into_iter()
        .filter(|canister_name| {
            !matches!(
                config
                    .get_config()
                    .get_remote_canister_id(canister_name, &network.name),
                Ok(Some(_))
            )
        })
        .collect();

    if some_canister.is_some() {
        info!(log, "Deploying: {}", canisters_to_deploy.join(" "));
    } else {
        info!(log, "Deploying all canisters.");
    }

    register_canisters(
        env,
        &canisters_to_build,
        &initial_canister_id_store,
        timeout,
        with_cycles,
        create_call_sender,
        &config,
    )
    .await?;

    build_canisters(env, &canisters_to_build, &config)?;

    install_canisters(
        env,
        &canisters_to_deploy,
        &initial_canister_id_store,
        &config,
        argument,
        argument_type,
        force_reinstall,
        upgrade_unchanged,
        timeout,
        call_sender,
    )
    .await?;

    info!(log, "Deployed canisters.");

    Ok(())
}

#[context("Failed to collect canisters and their dependencies.")]
fn canister_with_dependencies(
    config: &Config,
    some_canister: Option<&str>,
) -> DfxResult<Vec<String>> {
    let mut canister_names = config
        .get_config()
        .get_canister_names_with_dependencies(some_canister)?;
    canister_names.sort();
    Ok(canister_names)
}

#[context("Failed while trying to register all canisters.")]
async fn register_canisters(
    env: &dyn Environment,
    canister_names: &[String],
    canister_id_store: &CanisterIdStore,
    timeout: Duration,
    with_cycles: Option<&str>,
    call_sender: &CallSender,
    config: &Config,
) -> DfxResult {
    let canisters_to_create = canister_names
        .iter()
        .filter(|n| canister_id_store.find(n).is_none())
        .cloned()
        .collect::<Vec<String>>();
    if canisters_to_create.is_empty() {
        info!(env.get_logger(), "All canisters have already been created.");
    } else {
        info!(env.get_logger(), "Creating canisters...");
        for canister_name in &canisters_to_create {
            let config_interface = config.get_config();
            let compute_allocation =
                config_interface
                    .get_compute_allocation(canister_name)?
                    .map(|arg| {
                        ComputeAllocation::try_from(arg.parse::<u64>().unwrap())
                            .expect("Compute Allocation must be a percentage.")
                    });
            let memory_allocation =
                config_interface
                    .get_memory_allocation(canister_name)?
                    .map(|arg| {
                        MemoryAllocation::try_from(
                        u64::try_from(arg.parse::<Bytes>().unwrap().size()).unwrap(),
                    )
                    .expect(
                        "Memory allocation must be between 0 and 2^48 (i.e 256TB), inclusively.",
                    )
                    });
            let freezing_threshold =
                config_interface
                    .get_freezing_threshold(canister_name)?
                    .map(|arg| {
                        FreezingThreshold::try_from(
                            u128::try_from(arg.parse::<Bytes>().unwrap().size()).unwrap(),
                        )
                        .expect("Freezing threshold must be between 0 and 2^64-1, inclusively.")
                    });
            let controllers = None;
            create_canister(
                env,
                canister_name,
                timeout,
                with_cycles,
                call_sender,
                CanisterSettings {
                    controllers,
                    compute_allocation,
                    memory_allocation,
                    freezing_threshold,
                },
            )
            .await?;
        }
    }
    Ok(())
}

#[context("Failed to build call canisters.")]
fn build_canisters(env: &dyn Environment, canister_names: &[String], config: &Config) -> DfxResult {
    info!(env.get_logger(), "Building canisters...");
    let build_mode_check = false;
    let canister_pool = CanisterPool::load(env, build_mode_check, canister_names)?;

    canister_pool.build_or_fail(&BuildConfig::from_config(config)?)
}

#[allow(clippy::too_many_arguments)]
#[context("Failed while trying to install all canisters.")]
async fn install_canisters(
    env: &dyn Environment,
    canister_names: &[String],
    initial_canister_id_store: &CanisterIdStore,
    config: &Config,
    argument: Option<&str>,
    argument_type: Option<&str>,
    force_reinstall: bool,
    upgrade_unchanged: bool,
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    info!(env.get_logger(), "Installing canisters...");

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;

    let mut canister_id_store = CanisterIdStore::for_env(env)?;

    for canister_name in canister_names {
        let (install_mode, installed_module_hash) = if force_reinstall {
            (InstallMode::Reinstall, None)
        } else {
            match initial_canister_id_store.find(canister_name) {
                Some(canister_id) => {
                    match agent
                        .read_state_canister_info(canister_id, "module_hash", false)
                        .await
                    {
                        Ok(installed_module_hash) => {
                            (InstallMode::Upgrade, Some(installed_module_hash))
                        }
                        // If the canister is empty, this path does not exist.
                        // The replica doesn't support negative lookups, therefore if the canister
                        // is empty, the replica will return lookup_path([], Pruned _) = Unknown
                        Err(AgentError::LookupPathUnknown(_))
                        | Err(AgentError::LookupPathAbsent(_)) => (InstallMode::Install, None),
                        Err(x) => bail!(x),
                    }
                }
                None => (InstallMode::Install, None),
            }
        };

        let canister_id = canister_id_store.get(canister_name)?;
        let canister_info = CanisterInfo::load(config, canister_name, Some(canister_id))?;

        let maybe_path = canister_info.get_output_idl_path();
        let init_type = maybe_path.and_then(|path| get_candid_init_type(&path));
        let install_args = blob_from_arguments(argument, None, argument_type, &init_type)?;

        install_canister(
            env,
            agent,
            &mut canister_id_store,
            &canister_info,
            &install_args,
            install_mode,
            timeout,
            call_sender,
            installed_module_hash,
            upgrade_unchanged,
        )
        .await?;
    }

    Ok(())
}
