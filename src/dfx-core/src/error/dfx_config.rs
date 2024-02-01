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

    #[error("Failed to add dependencies for canister '{0}': {1}")]
    AddDependenciesFailed(String, AddDependenciesError),
}

#[derive(Error, Debug)]
pub enum GetComputeAllocationError {
    #[error("Failed to get compute allocation for canister '{0}': {1}")]
    GetComputeAllocationFailed(String, GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetFreezingThresholdError {
    #[error("Failed to get freezing threshold for canister '{0}': {1}")]
    GetFreezingThresholdFailed(String, GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetReservedCyclesLimitError {
    #[error("Failed to get reserved cycles limit for canister '{0}': {1}")]
    GetReservedCyclesLimitFailed(String, GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetMemoryAllocationError {
    #[error("Failed to get memory allocation for canister '{0}': {1}")]
    GetMemoryAllocationFailed(String, GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetPullCanistersError {
    #[error("Pull dependencies '{0}' and '{1}' have the same canister ID: {2}")]
    PullCanistersSameId(String, String, Principal),
}

#[derive(Error, Debug)]
pub enum GetRemoteCanisterIdError {
    #[error("Failed to figure out if canister '{0}' has a remote id on network '{1}': {2}")]
    GetRemoteCanisterIdFailed(Box<String>, Box<String>, GetCanisterConfigError),
}

#[derive(Error, Debug)]
pub enum GetSpecifiedIdError {
    #[error("Failed to get specified_id for canister '{0}': {1}")]
    GetSpecifiedIdFailed(String, GetCanisterConfigError),
}