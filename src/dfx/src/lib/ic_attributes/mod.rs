use crate::config::dfinity::ConfigInterface;
use crate::lib::error::DfxResult;

use humanize_rs::bytes::Bytes;
use ic_types::principal::Principal;
use ic_utils::interfaces::management_canister::attributes::{
    ComputeAllocation, FreezingThreshold, MemoryAllocation,
};
use std::convert::TryFrom;

pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<ComputeAllocation>,
    pub memory_allocation: Option<MemoryAllocation>,
    pub freezing_threshold: Option<FreezingThreshold>,
}

pub fn get_compute_allocation(
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

pub fn get_memory_allocation(
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

pub fn get_freezing_threshold(
    freezing_threshold: Option<String>,
    config_interface: &ConfigInterface,
    canister_name: &str,
) -> DfxResult<Option<FreezingThreshold>> {
    Ok(freezing_threshold
        .or(config_interface.get_freezing_threshold(canister_name)?)
        .map(|arg| {
            FreezingThreshold::try_from(arg.parse::<u128>().unwrap())
                .expect("Must be a value between 0 and 2^64-1 inclusive.")
        }))
}
