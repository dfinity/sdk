use crate::lib::builders::BuildConfig;
use crate::lib::canister_info::assets::AssetsCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::cycles_ledger_types::create_canister::SubnetSelection;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::wallet::{get_or_create_wallet_canister, GetOrCreateWalletCanisterError};
use crate::lib::installers::assets::prepare_assets_for_proposal;
use crate::lib::models::canister::CanisterPool;
use crate::lib::operations::canister::deploy_canisters::DeployMode::{
    ComputeEvidence, ForceReinstallSingleCanister, NormalDeploy, PrepareForProposal,
};
use crate::lib::operations::canister::motoko_playground::reserve_canister_with_playground;
use crate::lib::operations::canister::{create_canister, install_canister::install_canister};
use crate::util::{blob_from_arguments, get_candid_init_type};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::dfinity::Config;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::attributes::{
    ComputeAllocation, FreezingThreshold, MemoryAllocation, ReservedCyclesLimit,
};
use ic_utils::interfaces::management_canister::builders::InstallMode;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::info;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum DeployMode {
    NormalDeploy,
    ForceReinstallSingleCanister(String),
    PrepareForProposal(String),
    ComputeEvidence(String),
}

#[context("Failed while trying to deploy canisters.")]
pub async fn deploy_canisters(
    env: &dyn Environment,
    some_canister: Option<&str>,
    argument: Option<&str>,
    argument_type: Option<&str>,
    deploy_mode: &DeployMode,
    upgrade_unchanged: bool,
    with_cycles: Option<u128>,
    created_at_time: Option<u64>,
    specified_id: Option<Principal>,
    call_sender: &CallSender,
    from_subaccount: Option<Subaccount>,
    no_wallet: bool,
    skip_consent: bool,
    env_file: Option<PathBuf>,
    no_asset_upgrade: bool,
    subnet_selection: Option<SubnetSelection>,
) -> DfxResult {
    let log = env.get_logger();

    let config = env
        .get_config()
        .ok_or_else(|| anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))?;
    let initial_canister_id_store = env.get_canister_id_store()?;

    let pull_canisters_in_config = config.get_config().get_pull_canisters()?;
    if let Some(canister_name) = some_canister {
        if pull_canisters_in_config.contains_key(canister_name) {
            bail!(
                "{0} is a pull dependency. Please deploy it using `dfx deps deploy {0}`",
                canister_name
            );
        }
    }

    let canisters_to_load = canister_with_dependencies(&config, some_canister)?;

    let canisters_to_build = match deploy_mode {
        PrepareForProposal(canister_name) | ComputeEvidence(canister_name) => {
            vec![canister_name.clone()]
        }
        ForceReinstallSingleCanister(canister_name) => {
            // don't force-reinstall the dependencies too.
            vec![String::from(canister_name)]
        }
        NormalDeploy => canisters_to_load
            .clone()
            .into_iter()
            .filter(|canister_name| {
                !config
                    .get_config()
                    .is_remote_canister(canister_name, &env.get_network_descriptor().name)
                    .unwrap_or(false)
            })
            .collect(),
    };

    let canisters_to_install: Vec<String> = canisters_to_build
        .clone()
        .into_iter()
        .filter(|canister_name| !pull_canisters_in_config.contains_key(canister_name))
        .collect();

    if some_canister.is_some() {
        info!(log, "Deploying: {}", canisters_to_install.join(" "));
    } else {
        info!(log, "Deploying all canisters.");
    }
    if canisters_to_load
        .iter()
        .any(|canister| initial_canister_id_store.find(canister).is_none())
    {
        let proxy_sender;
        let create_call_sender = if no_wallet
            || specified_id.is_some()
            || matches!(call_sender, CallSender::Wallet(_))
            || env.get_network_descriptor().is_playground()
        {
            call_sender
        } else {
            match get_or_create_wallet_canister(
                env,
                env.get_network_descriptor(),
                env.get_selected_identity().expect("No selected identity"),
            )
            .await
            {
                Ok(wallet) => {
                    proxy_sender = CallSender::Wallet(*wallet.canister_id_());
                    &proxy_sender
                }
                Err(err) => match err {
                    GetOrCreateWalletCanisterError::NoWalletConfigured { .. } => call_sender,
                    _ => bail!(err),
                },
            }
        };
        register_canisters(
            env,
            &canisters_to_load,
            &initial_canister_id_store,
            with_cycles,
            specified_id,
            create_call_sender,
            from_subaccount,
            created_at_time,
            &config,
            subnet_selection,
        )
        .await?;
    } else {
        info!(env.get_logger(), "All canisters have already been created.");
    }

    let pool = build_canisters(
        env,
        &canisters_to_load,
        &canisters_to_build,
        &config,
        env_file.clone(),
    )
    .await?;

    match deploy_mode {
        NormalDeploy | ForceReinstallSingleCanister(_) => {
            let force_reinstall = matches!(deploy_mode, ForceReinstallSingleCanister(_));
            install_canisters(
                env,
                &canisters_to_install,
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
                no_asset_upgrade,
            )
            .await?;
            info!(log, "Deployed canisters.");
        }
        PrepareForProposal(canister_name) => {
            prepare_assets_for_commit(env, &initial_canister_id_store, &config, canister_name)
                .await?
        }
        ComputeEvidence(canister_name) => {
            compute_evidence(env, &initial_canister_id_store, &config, canister_name).await?
        }
    }

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
    with_cycles: Option<u128>,
    specified_id: Option<Principal>,
    call_sender: &CallSender,
    from_subaccount: Option<Subaccount>,
    created_at_time: Option<u64>,
    config: &Config,
    subnet_selection: Option<SubnetSelection>,
) -> DfxResult {
    let canisters_to_create = canister_names
        .iter()
        .filter(|n| canister_id_store.find(n).is_none())
        .cloned()
        .collect::<Vec<String>>();
    if canisters_to_create.is_empty() {
        info!(env.get_logger(), "All canisters have already been created.");
    } else if env.get_network_descriptor().is_playground() {
        info!(env.get_logger(), "Reserving canisters in playground...");
        for canister_name in &canisters_to_create {
            reserve_canister_with_playground(env, canister_name).await?;
        }
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
            let reserved_cycles_limit = config_interface
                .get_reserved_cycles_limit(canister_name)?
                .map(|arg| {
                    ReservedCyclesLimit::try_from(arg)
                        .expect("Reserved cycles limit must be between 0 and 2^128-1, inclusively.")
                });
            let controllers = None;
            create_canister(
                env,
                canister_name,
                with_cycles,
                specified_id,
                call_sender,
                from_subaccount,
                CanisterSettings {
                    controllers,
                    compute_allocation,
                    memory_allocation,
                    freezing_threshold,
                    reserved_cycles_limit,
                },
                created_at_time,
                subnet_selection.clone(),
            )
            .await?;
        }
    }
    Ok(())
}

#[context("Failed to build all canisters.")]
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

    let build_config =
        BuildConfig::from_config(config, env.get_network_descriptor().is_playground())?
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
    no_asset_upgrade: bool,
) -> DfxResult {
    info!(env.get_logger(), "Installing canisters...");

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

        let idl_path = canister_info.get_constructor_idl_path();
        let init_type = get_candid_init_type(&idl_path);
        let install_args =
            || blob_from_arguments(Some(env), argument, None, argument_type, &init_type);

        install_canister(
            env,
            &mut canister_id_store,
            canister_id,
            Some(&canister_info),
            None,
            install_args,
            install_mode,
            call_sender,
            upgrade_unchanged,
            Some(&pool),
            skip_consent,
            env_file,
            no_asset_upgrade,
        )
        .await?;
    }

    Ok(())
}

#[context("Failed to prepare assets for commit.")]
async fn prepare_assets_for_commit(
    env: &dyn Environment,
    canister_id_store: &CanisterIdStore,
    config: &Config,
    canister_name: &str,
) -> DfxResult {
    let canister_id = canister_id_store.get(canister_name)?;
    let canister_info = CanisterInfo::load(config, canister_name, Some(canister_id))?;

    if !canister_info.is_assets() {
        bail!(
            "Expected canister {} to be an asset canister.",
            canister_name
        );
    }

    let agent = env.get_agent();

    prepare_assets_for_proposal(&canister_info, agent, env.get_logger()).await?;

    Ok(())
}

#[context("Failed to compute evidence.")]
async fn compute_evidence(
    env: &dyn Environment,
    canister_id_store: &CanisterIdStore,
    config: &Config,
    canister_name: &str,
) -> DfxResult {
    let canister_id = canister_id_store.get(canister_name)?;
    let canister_info = CanisterInfo::load(config, canister_name, Some(canister_id))?;

    if !canister_info.is_assets() {
        bail!(
            "Expected canister {} to be an asset canister.",
            canister_name
        );
    }

    let agent = env.get_agent();

    let assets_canister_info = canister_info.as_info::<AssetsCanisterInfo>()?;
    let source_paths = assets_canister_info.get_source_paths();
    let source_paths: Vec<&Path> = source_paths.iter().map(|p| p.as_path()).collect::<_>();

    let canister_id = canister_info
        .get_canister_id()
        .context("Could not find canister ID.")?;

    let canister = ic_utils::Canister::builder()
        .with_agent(agent)
        .with_canister_id(canister_id)
        .build()
        .context("Failed to build asset canister caller.")?;

    let evidence = ic_asset::compute_evidence(&canister, &source_paths, env.get_logger()).await?;
    println!("{}", evidence);

    Ok(())
}
