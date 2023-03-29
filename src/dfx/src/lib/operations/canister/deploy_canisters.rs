use crate::lib::builders::BuildConfig;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister::CanisterPool;
use crate::lib::operations::canister::{create_canister, install_canister};
use crate::util::{blob_from_arguments, get_candid_init_type};
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::dfinity::Config;

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::attributes::{
    ComputeAllocation, FreezingThreshold, MemoryAllocation,
};
use ic_utils::interfaces::management_canister::builders::InstallMode;
use slog::info;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

#[context("Failed while trying to deploy canisters.")]
pub async fn deploy_canisters(
    env: &dyn Environment,
    some_canister: Option<&str>,
    argument: Option<&str>,
    argument_type: Option<&str>,
    force_reinstall: bool,
    upgrade_unchanged: bool,
    with_cycles: Option<&str>,
    specified_id: Option<Principal>,
    call_sender: &CallSender,
    create_call_sender: &CallSender,
    skip_consent: bool,
    env_file: Option<PathBuf>,
    assets_upgrade: bool,
) -> DfxResult {
    let log = env.get_logger();

    let config = env
        .get_config()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;
    let initial_canister_id_store = env.get_canister_id_store()?;

    let network = env.get_network_descriptor();

    let canisters_to_load = canister_with_dependencies(&config, some_canister)?;

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
        canisters_to_load
            .clone()
            .into_iter()
            .filter(|canister_name| {
                !config
                    .get_config()
                    .is_remote_canister(canister_name, &env.get_network_descriptor().name)
                    .unwrap_or(false)
            })
            .collect()
    };

    if some_canister.is_some() {
        info!(log, "Deploying: {}", canisters_to_deploy.join(" "));
    } else {
        info!(log, "Deploying all canisters.");
    }

    register_canisters(
        env,
        &canisters_to_load,
        &initial_canister_id_store,
        with_cycles,
        specified_id,
        create_call_sender,
        &config,
    )
    .await?;

    let pool = build_canisters(
        env,
        &canisters_to_load,
        &canisters_to_deploy,
        &config,
        env_file.clone(),
    )
    .await?;

    install_canisters(
        env,
        &canisters_to_deploy,
        &initial_canister_id_store,
        &config,
        argument,
        argument_type,
        force_reinstall,
        upgrade_unchanged,
        call_sender,
        pool,
        skip_consent,
        env_file.as_deref(),
        assets_upgrade,
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

/// Creates canisters that have not been created yet.
#[context("Failed while trying to register all canisters.")]
async fn register_canisters(
    env: &dyn Environment,
    canister_names: &[String],
    canister_id_store: &CanisterIdStore,
    with_cycles: Option<&str>,
    specified_id: Option<Principal>,
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
            let compute_allocation = config_interface
                .get_compute_allocation(canister_name)?
                .map(|arg| {
                    ComputeAllocation::try_from(arg)
                        .context("Compute Allocation must be a percentage.")
                })
                .transpose()?;
            let memory_allocation = config_interface
                .get_memory_allocation(canister_name)?
                .map(|arg| {
                    u64::try_from(arg.get_bytes())
                        .map_err(|e| anyhow!(e))
                        .and_then(|n| Ok(MemoryAllocation::try_from(n)?))
                        .context(
                            "Memory allocation must be between 0 and 2^48 (i.e 256TB), inclusively.",
                        )
                })
                .transpose()?;
            let freezing_threshold =
                config_interface
                    .get_freezing_threshold(canister_name)?
                    .map(|arg| {
                        FreezingThreshold::try_from(arg.as_secs())
                            .expect("Freezing threshold must be between 0 and 2^64-1, inclusively.")
                    });
            let controllers = None;
            create_canister(
                env,
                canister_name,
                with_cycles,
                specified_id,
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
async fn build_canisters(
    env: &dyn Environment,
    referenced_canisters: &[String],
    canisters_to_build: &[String],
    config: &Config,
    env_file: Option<PathBuf>,
) -> DfxResult<CanisterPool> {
    let log = env.get_logger();
    info!(log, "Building canisters...");
    let build_mode_check = false;
    let canister_pool = CanisterPool::load(env, build_mode_check, referenced_canisters)?;
    let build_config = BuildConfig::from_config(config)?
        .with_canisters_to_build(canisters_to_build.into())
        .with_env_file(env_file);
    canister_pool.build_or_fail(log, &build_config).await?;
    Ok(canister_pool)
}

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
    call_sender: &CallSender,
    pool: CanisterPool,
    skip_consent: bool,
    env_file: Option<&Path>,
    assets_upgrade: bool,
) -> DfxResult {
    info!(env.get_logger(), "Installing canisters...");

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;

    let mut canister_id_store = env.get_canister_id_store()?;

    for canister_name in canister_names {
        let install_mode = if force_reinstall {
            Some(InstallMode::Reinstall)
        } else {
            match initial_canister_id_store.find(canister_name) {
                Some(_) => None,
                None => Some(InstallMode::Install),
            }
        };

        let canister_id = canister_id_store.get(canister_name)?;
        let canister_info = CanisterInfo::load(config, canister_name, Some(canister_id))?;

        let idl_path = canister_info.get_build_idl_path();
        let init_type = get_candid_init_type(&idl_path);
        let install_args = || blob_from_arguments(argument, None, argument_type, &init_type);

        install_canister(
            env,
            agent,
            &mut canister_id_store,
            &canister_info,
            &install_args,
            install_mode,
            call_sender,
            upgrade_unchanged,
            Some(&pool),
            skip_consent,
            env_file,
            assets_upgrade,
        )
        .await?;
    }

    Ok(())
}
