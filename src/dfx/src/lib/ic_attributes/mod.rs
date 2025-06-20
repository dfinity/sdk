use crate::lib::canister_logs::log_visibility::LogVisibilityOpt;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use anyhow::{anyhow, Context, Error};
use byte_unit::Byte;
use candid::Principal;
use dfx_core::config::model::dfinity::ConfigInterface;
use fn_error_context::context;
use ic_utils::interfaces::management_canister::{
    attributes::{ComputeAllocation, FreezingThreshold, MemoryAllocation, ReservedCyclesLimit},
    builders::WasmMemoryLimit,
    LogVisibility, StatusCallResult,
};
use num_traits::ToPrimitive;
use std::convert::TryFrom;

#[derive(Default, Debug, Clone)]
pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<ComputeAllocation>,
    pub memory_allocation: Option<MemoryAllocation>,
    pub freezing_threshold: Option<FreezingThreshold>,
    pub reserved_cycles_limit: Option<ReservedCyclesLimit>,
    pub wasm_memory_limit: Option<WasmMemoryLimit>,
    pub wasm_memory_threshold: Option<WasmMemoryLimit>,
    pub log_visibility: Option<LogVisibility>,
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
            wasm_memory_limit: value
                .wasm_memory_limit
                .map(u64::from)
                .map(candid::Nat::from),
            wasm_memory_threshold: value
                .wasm_memory_threshold
                .map(u64::from)
                .map(candid::Nat::from),
            log_visibility: value.log_visibility,
        }
    }
}

impl TryFrom<ic_utils::interfaces::management_canister::builders::CanisterSettings>
    for CanisterSettings
{
    type Error = Error;
    fn try_from(
        value: ic_utils::interfaces::management_canister::builders::CanisterSettings,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            controllers: value.controllers,
            compute_allocation: value
                .compute_allocation
                .and_then(|alloc| alloc.0.to_u8())
                .map(|alloc| {
                    ComputeAllocation::try_from(alloc)
                        .context("Compute allocation must be a percentage.")
                })
                .transpose()?,
            memory_allocation: value
                .memory_allocation
                .and_then(|alloc| alloc.0.to_u64())
                .map(|alloc| {
                    MemoryAllocation::try_from(alloc).context(
                        "Memory allocation must be between 0 and 2^48 (i.e 256TB), inclusively.",
                    )
                })
                .transpose()?,
            freezing_threshold: value
                .freezing_threshold
                .and_then(|threshold| threshold.0.to_u64())
                .map(|threshold| {
                    FreezingThreshold::try_from(threshold)
                        .context("Freezing threshold must be between 0 and 2^64-1, inclusively.")
                })
                .transpose()?,
            reserved_cycles_limit: value
                .reserved_cycles_limit
                .and_then(|limit| limit.0.to_u128())
                .map(|limit| {
                    ReservedCyclesLimit::try_from(limit).context(
                        "Reserved cycles limit must be between 0 and 2^128-1, inclusively.",
                    )
                })
                .transpose()?,
            wasm_memory_limit: value
                .wasm_memory_limit
                .and_then(|limit| limit.0.to_u64())
                .map(|limit| {
                    WasmMemoryLimit::try_from(limit)
                        .context("Wasm memory limit must be between 0 and 2^48-1, inclusively.")
                })
                .transpose()?,
            wasm_memory_threshold: value
                .wasm_memory_threshold
                .and_then(|limit| limit.0.to_u64())
                .map(|limit| {
                    WasmMemoryLimit::try_from(limit)
                        .context("Wasm memory threshold msut be between 0 and 2^48-1, inclusively.")
                })
                .transpose()?,
            log_visibility: value.log_visibility,
        })
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

pub fn get_wasm_memory_limit(
    wasm_memory_limit: Option<Byte>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<WasmMemoryLimit>> {
    let wasm_memory_limit = match (wasm_memory_limit, config_interface, canister_name) {
        (Some(memory_limit), _, _) => Some(memory_limit),
        (None, Some(config_interface), Some(canister_name)) => {
            config_interface.get_wasm_memory_limit(canister_name)?
        }
        _ => None,
    };
    wasm_memory_limit
        .map(|arg| {
            u64::try_from(arg.get_bytes())
                .map_err(|e| anyhow!(e))
                .and_then(|n| Ok(WasmMemoryLimit::try_from(n)?))
                .context("Wasm memory limit must be between 0 and 2^48 (i.e 256TB), inclusively.")
        })
        .transpose()
}
pub fn get_wasm_memory_threshold(
    wasm_memory_threshold: Option<Byte>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<WasmMemoryLimit>> {
    let wasm_memory_threshold = match (wasm_memory_threshold, config_interface, canister_name) {
        (Some(memory_threshold), _, _) => Some(memory_threshold),
        (None, Some(config_interface), Some(canister_name)) => {
            config_interface.get_wasm_memory_threshold(canister_name)?
        }
        _ => None,
    };
    wasm_memory_threshold
        .map(|arg| {
            u64::try_from(arg.get_bytes())
                .map_err(|e| anyhow!(e))
                .and_then(|n| Ok(WasmMemoryLimit::try_from(n)?))
                .context("Wasm memory limit must be between 0 and 2^48 (i.e 256TB), inclusively.")
        })
        .transpose()
}

pub fn get_log_visibility(
    env: &dyn Environment,
    log_visibility: Option<&LogVisibilityOpt>,
    current_settings: Option<&StatusCallResult>,
    config_interface: Option<&ConfigInterface>,
    canister_name: Option<&str>,
) -> DfxResult<Option<LogVisibility>> {
    let log_visibility = match (log_visibility, config_interface, canister_name) {
        (Some(log_visibility), _, _) => {
            Some(log_visibility.to_log_visibility(env, current_settings)?)
        }
        (None, Some(config_interface), Some(canister_name)) => {
            config_interface.get_log_visibility(canister_name)?
        }
        _ => None,
    };
    Ok(log_visibility)
}
