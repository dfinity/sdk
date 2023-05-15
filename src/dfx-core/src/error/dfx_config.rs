use candid::Principal;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DfxConfigError {
    #[error("Circular canister dependencies: {}", _0.join(" -> "))]
    CanisterCircularDependency(Vec<String>),

    #[error("Canister '{0}' not found in dfx.json.")]
    CanisterNotFound(String),

    #[error("No canisters in the configuration file.")]
    CanistersFieldDoesNotExist(),

    #[error("Failed to get canisters with their dependencies (for {}): {1}", _0.as_deref().unwrap_or("all canisters"))]
    GetCanistersWithDependenciesFailed(Option<String>, Box<DfxConfigError>),

    #[error("Failed to get compute allocation for canister '{0}': {1}")]
    GetComputeAllocationFailed(String, Box<DfxConfigError>),

    #[error("Failed to get freezing threshold for canister '{0}': {1}")]
    GetFreezingThresholdFailed(String, Box<DfxConfigError>),

    #[error("Failed to get memory allocation for canister '{0}': {1}")]
    GetMemoryAllocationFailed(String, Box<DfxConfigError>),

    #[error("Failed to figure out if canister '{0}' has a remote id on network '{1}': {2}")]
    GetRemoteCanisterIdFailed(Box<String>, Box<String>, Box<DfxConfigError>),

    #[error("Pull dependencies '{0}' and '{1}' have the same canister ID: {2}")]
    PullCanistersSameId(String, String, Principal),
}
