use crate::config::dfinity::ConfigInterface;
use crate::lib::error::DfxResult;

use anyhow::{anyhow, Context};
use byte_unit::Byte;
use candid::Principal;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::attributes::{
    ComputeAllocation, FreezingThreshold, MemoryAllocation,
};
use std::convert::TryFrom;

#[derive(Default)]
pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<ComputeAllocation>,
    pub memory_allocation: Option<MemoryAllocation>,
    pub freezing_threshold: Option<FreezingThreshold>,
}

#[context("Failed to get compute allocation.")]
pub fn get_compute_allocation(
    compute_allocation: Option<String>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<ComputeAllocation>> {
    let compute_allocation = match (compute_allocation, config_interface, canister_name) {
        (Some(compute_allocation), _, _) => Some(compute_allocation.parse::<u64>()?),
        (None, Some(config_interface), Some(canister_name)) => {
            config_interface.get_compute_allocation(canister_name)? as _
        }
        _ => None,
    };
    compute_allocation
        .map(|arg| {
            ComputeAllocation::try_from(arg).context("Compute Allocation must be a percentage.")
        })
        .transpose()
}

#[context("Failed to get memory allocation.")]
pub fn get_memory_allocation(
    memory_allocation: Option<String>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<MemoryAllocation>> {
    let memory_allocation = match (memory_allocation, config_interface, canister_name) {
        (Some(memory_allocation), _, _) => Some(memory_allocation.parse::<Byte>()?),
        (None, Some(config_interface), Some(canister_name)) => {
            config_interface.get_memory_allocation(canister_name)?
        }
        _ => None,
    };
    memory_allocation
        .map(|arg| {
            u64::try_from(arg.get_bytes())
                .map_err(|e| anyhow!(e))
                .and_then(|n| Ok(MemoryAllocation::try_from(n)?))
                .context("Memory allocation must be between 0 and 2^48 (i.e 256TB), inclusively.")
        })
        .transpose()
}

#[context("Failed to get freezing threshold.")]
pub fn get_freezing_threshold(
    freezing_threshold: Option<String>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<FreezingThreshold>> {
    let freezing_threshold = match (freezing_threshold, config_interface, canister_name) {
        (Some(freezing_threshold), _, _) => Some(freezing_threshold.parse::<u64>()?),
        (None, Some(config_interface), Some(canister_name)) => config_interface
            .get_freezing_threshold(canister_name)?
            .map(|dur| dur.as_secs()),
        _ => None,
    };
    freezing_threshold
        .map(|arg| {
            FreezingThreshold::try_from(arg)
                .context("Must be a duration between 0 and 2^64-1 inclusive.")
        })
        .transpose()
}
