use crate::config::dfinity::ConfigInterface;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::install_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::{compute_allocation_validator, memory_allocation_validator};
use crate::util::{blob_from_arguments, expiry_duration, get_candid_init_type};

use anyhow::{anyhow, bail};
use clap::Clap;
use humanize_rs::bytes::Bytes;
use ic_utils::interfaces::management_canister::{ComputeAllocation, InstallMode, MemoryAllocation};
use std::convert::TryFrom;
use std::str::FromStr;

/// Deploys compiled code as a canister on the Internet Computer.
#[derive(Clap, Clone)]
#[clap(name("install"))]
pub struct CanisterInstallOpts {
    /// Specifies the canister name to deploy. You must specify either canister name or the --all option.
    canister_name: Option<String>,

    /// Deploys all canisters configured in the project dfx.json files.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,

    /// Specifies not to wait for the result of the call to be returned by polling the replica. Instead return a response ID.
    #[clap(long)]
    async_call: bool,

    /// Specifies the type of deployment. You can set the canister deployment modes to install, reinstall, or upgrade.
    #[clap(long, short('m'), default_value("install"),
        possible_values(&["install", "reinstall", "upgrade"]))]
    mode: String,

    /// Specifies the argument to pass to the method.
    #[clap(long)]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[clap(long, requires("argument"), possible_values(&["idl", "raw"]))]
    argument_type: Option<String>,

    /// Specifies the canister's compute allocation. This should be a percent in the range [0..100]
    #[clap(long, short('c'), validator(compute_allocation_validator))]
    compute_allocation: Option<String>,

    /// Specifies how much memory the canister is allowed to use in total.
    /// This should be a value in the range [0..256 TB]
    #[clap(long, validator(memory_allocation_validator))]
    memory_allocation: Option<String>,
}

fn get_compute_allocation(
    compute_allocation: Option<String>,
    config_interface: &ConfigInterface,
    canister_name: &str,
) -> DfxResult<Option<ComputeAllocation>> {
    Ok(compute_allocation
        .or(config_interface.get_compute_allocation(canister_name)?)
        .map(|arg| {
            ComputeAllocation::try_from(arg.parse::<u64>().unwrap())
                .expect("Compute Allocation must be a percentage.")
        }))
}

fn get_memory_allocation(
    memory_allocation: Option<String>,
    config_interface: &ConfigInterface,
    canister_name: &str,
) -> DfxResult<Option<MemoryAllocation>> {
    Ok(memory_allocation
        .or(config_interface.get_memory_allocation(canister_name)?)
        .map(|arg| {
            MemoryAllocation::try_from(u64::try_from(arg.parse::<Bytes>().unwrap().size()).unwrap())
                .expect("Memory allocation must be between 0 and 2^48 (i.e 256TB), inclusively.")
        }))
}

pub async fn exec(env: &dyn Environment, opts: CanisterInstallOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;

    let config_interface = config.get_config();
    let mode = InstallMode::from_str(opts.mode.as_str()).map_err(|err| anyhow!(err))?;
    let canister_id_store = CanisterIdStore::for_env(env)?;

    if let Some(canister_name) = opts.canister_name.as_deref() {
        let canister_id = canister_id_store.get(canister_name)?;
        let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;

        let maybe_path = canister_info.get_output_idl_path();
        let init_type = maybe_path.and_then(|path| get_candid_init_type(&path));
        let arguments = opts.argument.as_deref();
        let arg_type = opts.argument_type.as_deref();
        let install_args = blob_from_arguments(arguments, arg_type, &init_type)?;

        let compute_allocation = get_compute_allocation(
            opts.compute_allocation.clone(),
            config_interface,
            canister_name,
        )?;
        let memory_allocation = get_memory_allocation(
            opts.memory_allocation.clone(),
            config_interface,
            canister_name,
        )?;

        install_canister(
            env,
            &agent,
            &canister_info,
            &install_args,
            compute_allocation,
            mode,
            memory_allocation,
            timeout,
        )
        .await
    } else if opts.all {
        // Install all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                let canister_id = canister_id_store.get(canister_name)?;
                let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;

                let install_args = [];

                let compute_allocation = get_compute_allocation(
                    opts.compute_allocation.clone(),
                    config_interface,
                    canister_name,
                )?;
                let memory_allocation = get_memory_allocation(
                    opts.memory_allocation.clone(),
                    config_interface,
                    canister_name,
                )?;

                install_canister(
                    env,
                    &agent,
                    &canister_info,
                    &install_args,
                    compute_allocation,
                    mode,
                    memory_allocation,
                    timeout,
                )
                .await?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
