use crate::lib::error::DfxResult;
use anyhow::{anyhow, Context};
use byte_unit::Byte;
use candid::Principal;
use dfx_core::config::model::dfinity::ConfigInterface;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::attributes::{
    ComputeAllocation, FreezingThreshold, MemoryAllocation, ReservedCyclesLimit,
};
use std::convert::TryFrom;

#[derive(Default, Debug)]
pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<ComputeAllocation>,
    pub memory_allocation: Option<MemoryAllocation>,
    pub freezing_threshold: Option<FreezingThreshold>,
    pub reserved_cycles_limit: Option<ReservedCyclesLimit>,
}

impl From<CanisterSettings>
    for ic_utils::interfaces::management_canister::builders::CanisterSettings
{
    fn from(value: CanisterSettings) -> Self {
        Self {
            controllers: value.controllers,
            compute_allocation: value
                .compute_allocation
                .map(u8::from)
                .map(candid::Nat::from),
            memory_allocation: value
                .memory_allocation
                .map(u64::from)
                .map(candid::Nat::from),
            freezing_threshold: value
                .freezing_threshold
                .map(u64::from)
                .map(candid::Nat::from),
            reserved_cycles_limit: value
                .reserved_cycles_limit
                .map(u128::from)
                .map(candid::Nat::from),
        }
    }
}

#[context("Failed to get compute allocation.")]
pub fn get_compute_allocation(
    compute_allocation: Option<u64>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<ComputeAllocation>> {
    let compute_allocation = match (compute_allocation, config_interface, canister_name) {
        (Some(compute_allocation), _, _) => Some(compute_allocation),
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
    memory_allocation: Option<Byte>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<MemoryAllocation>> {
    let memory_allocation = match (memory_allocation, config_interface, canister_name) {
        (Some(memory_allocation), _, _) => Some(memory_allocation),
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
    freezing_threshold: Option<u64>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<FreezingThreshold>> {
    let freezing_threshold = match (freezing_threshold, config_interface, canister_name) {
        (Some(freezing_threshold), _, _) => Some(freezing_threshold),
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

#[context("Failed to get reserved cycles limit")]
pub fn get_reserved_cycles_limit(
    reserved_cycles_limit: Option<u128>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<ReservedCyclesLimit>> {
    let reserved_cycles_limit = match (reserved_cycles_limit, config_interface, canister_name) {
        (Some(reserved_cycles_limit), _, _) => Some(reserved_cycles_limit),
        (None, Some(config_interface), Some(canister_name)) => {
            config_interface.get_reserved_cycles_limit(canister_name)?
        }
        _ => None,
    };
    reserved_cycles_limit
        .map(|arg| {
            ReservedCyclesLimit::try_from(arg)
                .context("Must be a limit between 0 and 2^128-1 inclusive.")
        })
        .transpose()
}
