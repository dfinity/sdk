use crate::config::dfinity::ConfigInterface;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::operations::canister::create_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::{compute_allocation_validator, memory_allocation_validator};
use crate::util::expiry_duration;

use anyhow::{anyhow, bail};
use clap::Clap;
use humanize_rs::bytes::Bytes;
use ic_types::principal::Principal as CanisterId;
use ic_utils::interfaces::management_canister::attributes::{ComputeAllocation, MemoryAllocation};
use std::convert::TryFrom;

/// Creates an empty canister on the Internet Computer and
/// associates the Internet Computer assigned Canister ID to the canister name.
#[derive(Clap)]
pub struct CanisterCreateOpts {
    /// Specifies the canister name. Either this or the --all flag are required.
    canister_name: Option<String>,

    /// Creates all canisters configured in dfx.json.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,

    /// Specifies the identity name or the principal of the new controller.
    controller: Option<String>,

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

pub async fn exec(env: &dyn Environment, opts: CanisterCreateOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;
    let config_interface = config.get_config();

    let controller = if let Some(controller) = opts.controller {
        match CanisterId::from_text(controller.clone()) {
            Ok(principal) => Some(principal),
            Err(_) => Some(
                IdentityManager::new(env)?
                    .instantiate_identity_from_name(controller.as_str())?
                    .sender()
                    .map_err(|err| anyhow!(err))?,
            ),
        }
    } else {
        None
    };

    if let Some(canister_name) = opts.canister_name.as_deref() {
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
        create_canister(
            env,
            canister_name,
            timeout,
            controller,
            compute_allocation,
            memory_allocation,
        )
        .await
    } else if opts.all {
        // Create all canisters.
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
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
                create_canister(
                    env,
                    canister_name,
                    timeout,
                    controller.clone(),
                    compute_allocation,
                    memory_allocation,
                )
                .await?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
