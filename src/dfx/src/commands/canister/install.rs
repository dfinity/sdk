use crate::config::dfinity::ConfigInterface;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::install_canister;
use crate::util::{blob_from_arguments, expiry_duration, get_candid_init_type};
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use humanize_rs::bytes::{Bytes, Unit};
use ic_utils::interfaces::management_canister::{ComputeAllocation, InstallMode, MemoryAllocation};
use std::convert::TryFrom;
use std::str::FromStr;
use tokio::runtime::Runtime;

/// Deploys compiled code as a canister on the Internet Computer.
#[derive(Clap, Clone)]
pub struct CanisterInstallOpts {
    /// Specifies the canister name to deploy. You must specify either canister name or the --all option.
    #[clap(required_unless_present("all"))]
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

pub fn construct() -> App<'static> {
    CanisterInstallOpts::into_app().name("install")
}

fn compute_allocation_validator(compute_allocation: &str) -> Result<(), String> {
    if let Ok(num) = compute_allocation.parse::<u64>() {
        if num <= 100 {
            return Ok(());
        }
    }
    Err("Must be a percent between 0 and 100".to_string())
}

fn memory_allocation_validator(memory_allocation: &str) -> Result<(), String> {
    let limit = Bytes::new(256, Unit::TByte).map_err(|_| "Parse Overflow.")?;
    if let Ok(bytes) = memory_allocation.parse::<Bytes>() {
        if bytes.size() <= limit.size() {
            return Ok(());
        }
    }
    Err("Must be a value between 0..256 TB inclusive.".to_string())
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

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CanisterInstallOpts = CanisterInstallOpts::from_arg_matches(args);
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let timeout = expiry_duration();

    let agent = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let config_interface = config.get_config();

    let mode = InstallMode::from_str(opts.mode.as_str())?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    let canister_id_store = CanisterIdStore::for_env(env)?;

    if let Some(canister_name) = opts.canister_name.as_deref() {
        let canister_id = canister_id_store.get(canister_name)?;
        let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;

        let maybe_path = canister_info.get_output_idl_path();
        let init_type = maybe_path.and_then(|path| get_candid_init_type(&path));
        let arguments = opts.argument.as_deref();
        let arg_type = opts.argument_type.as_deref();
        let install_args = blob_from_arguments(arguments, arg_type, &init_type)?;

        let compute_allocation =
            get_compute_allocation(opts.compute_allocation, config_interface, canister_name)?;
        let memory_allocation =
            get_memory_allocation(opts.memory_allocation, config_interface, canister_name)?;

        runtime.block_on(install_canister(
            env,
            &agent,
            &canister_info,
            &install_args,
            compute_allocation,
            mode,
            memory_allocation,
            timeout,
        ))?;
        Ok(())
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

                runtime.block_on(install_canister(
                    env,
                    &agent,
                    &canister_info,
                    &install_args,
                    compute_allocation,
                    mode,
                    memory_allocation,
                    timeout,
                ))?;
            }
        }
        Ok(())
    } else {
        Err(DfxError::CanisterNameMissing())
    }
}
