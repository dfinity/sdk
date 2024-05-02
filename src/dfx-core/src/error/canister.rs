use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanisterBuilderError {
    #[error("Failed to construct wallet canister caller")]
    WalletCanisterCaller(#[source] ic_agent::AgentError),

    #[error("Failed to build call sender")]
    CallSenderBuildError(#[source] ic_agent::AgentError),
}

#[derive(Error, Debug)]
pub enum CanisterInstallError {
    #[error("Refusing to install canister without approval")]
    UserConsent(#[source] crate::error::cli::UserConsent),

    #[error(transparent)]
    CanisterBuilderError(#[from] CanisterBuilderError),

    #[error("Failed during wasm installation call")]
    InstallWasmError(#[source] ic_agent::AgentError),
}
