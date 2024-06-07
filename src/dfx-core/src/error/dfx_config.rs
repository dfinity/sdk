use candid::Principal;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddDependenciesError {
    #[error("Circular canister dependencies: {}", _0.join(" -> "))]
    CanisterCircularDependency(Vec<String>),

    #[error("Canister '{0}' not found in dfx.json.")]
    CanisterNotFound(String),
}

#[derive(Error, Debug)]
pub enum GetCanisterConfigError {
    #[error("No canisters in the configuration file.")]
    CanistersFieldDoesNotExist(),

    #[error("Canister '{0}' not found in dfx.json.")]
    CanisterNotFound(String),
}

#[derive(Error, Debug)]
pub enum GetCanisterNamesWithDependenciesError {
    #[error("No canisters in the configuration file.")]
    CanistersFieldDoesNotExist(),

    #[error("Failed to add dependencies for canister '{0}'")]
    AddDependenciesFailed(String, #[source] AddDependenciesError),
}

#[derive(Error, Debug)]
pub enum GetComputeAllocationError {
    #[error("Failed to get compute allocation for canister '{0}'")]
    GetComputeAllocationFailed(String, #[source] GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetFreezingThresholdError {
    #[error("Failed to get freezing threshold for canister '{0}'")]
    GetFreezingThresholdFailed(String, #[source] GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetReservedCyclesLimitError {
    #[error("Failed to get reserved cycles limit for canister '{0}'")]
    GetReservedCyclesLimitFailed(String, #[source] GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetMemoryAllocationError {
    #[error("Failed to get memory allocation for canister '{0}'")]
    GetMemoryAllocationFailed(String, #[source] GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetWasmMemoryLimitError {
    #[error("Failed to get Wasm memory limit for canister '{0}'")]
    GetWasmMemoryLimitFailed(String, #[source] GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetPullCanistersError {
    #[error("Pull dependencies '{0}' and '{1}' have the same canister ID: {2}")]
    PullCanistersSameId(String, String, Principal),
}

#[derive(Error, Debug)]
pub enum GetRemoteCanisterIdError {
    #[error("Failed to figure out if canister '{0}' has a remote id on network '{1}'")]
    GetRemoteCanisterIdFailed(Box<String>, Box<String>, #[source] GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetSpecifiedIdError {
    #[error("Failed to get specified_id for canister '{0}'")]
    GetSpecifiedIdFailed(String, #[source] GetCanisterConfigError),
}
