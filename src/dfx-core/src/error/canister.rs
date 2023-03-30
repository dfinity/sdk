use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanisterBuilderError {
    #[error("Failed to construct wallet canister caller: {0}")]
    WalletCanisterCaller(ic_agent::AgentError),
}
