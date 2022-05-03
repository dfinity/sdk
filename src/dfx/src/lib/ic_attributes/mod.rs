use crate::config::dfinity::ConfigInterface;
use crate::lib::error::DfxResult;

use humanize_rs::bytes::Bytes;
use ic_types::principal::Principal;
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

pub fn get_compute_allocation(
    compute_allocation: Option<String>,
    config_interface: &ConfigInterface,
    canister_name: Option<&str>,
) -> DfxResult<Option<ComputeAllocation>> {
    let compute_allocation = match (compute_allocation, canister_name) {
        (Some(compute_allocation), _) => Some(compute_allocation),
        (None, Some(canister_name)) => config_interface.get_compute_allocation(canister_name)?,
        (None, None) => None,
    };
    Ok(compute_allocation.map(|arg| {
        ComputeAllocation::try_from(arg.parse::<u64>().unwrap())
            .expect("Compute Allocation must be a percentage.")
    }))
}

pub fn get_memory_allocation(
    memory_allocation: Option<String>,
    config_interface: &ConfigInterface,
    canister_name: Option<&str>,
) -> DfxResult<Option<MemoryAllocation>> {
    let memory_allocation = match (memory_allocation, canister_name) {
        (Some(memory_allocation), _) => Some(memory_allocation),
        (None, Some(canister_name)) => config_interface.get_memory_allocation(canister_name)?,
        (None, None) => None,
    };
    Ok(memory_allocation.map(|arg| {
        MemoryAllocation::try_from(u64::try_from(arg.parse::<Bytes>().unwrap().size()).unwrap())
            .expect("Memory allocation must be between 0 and 2^48 (i.e 256TB), inclusively.")
    }))
}

pub fn get_freezing_threshold(
    freezing_threshold: Option<String>,
    config_interface: &ConfigInterface,
    canister_name: Option<&str>,
) -> DfxResult<Option<FreezingThreshold>> {
    let freezing_threshold = match (freezing_threshold, canister_name) {
        (Some(freezing_threshold), _) => Some(freezing_threshold),
        (None, Some(canister_name)) => config_interface.get_freezing_threshold(canister_name)?,
        (None, None) => None,
    };
    Ok(freezing_threshold.map(|arg| {
        FreezingThreshold::try_from(arg.parse::<u128>().unwrap())
            .expect("Must be a value between 0 and 2^64-1 inclusive.")
    }))
}
