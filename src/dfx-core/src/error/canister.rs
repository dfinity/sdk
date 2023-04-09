use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanisterBuilderError {
    #[error("Failed to construct wallet canister caller: {0}")]
    WalletCanisterCaller(ic_agent::AgentError),

    #[error("Failed to build call sender: {0}")]
    CallSenderBuildError(ic_agent::AgentError),
}

#[derive(Error, Debug)]
pub enum CanisterInstallError {
    #[error("Refusing to install canister without approval: {0}")]
    UserConsent(crate::error::cli::UserConsent),

    #[error(transparent)]
    CanisterBuilderError(#[from] CanisterBuilderError),

    #[error("Failed during wasm installation call: {0}")]
    InstallWasmError(ic_agent::AgentError),
}
