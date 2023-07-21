use crate::lib::ledger_types::NotifyError;
use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyCreateCanisterError {
    #[error("Failed when calling notify_create_canister: {0:#}")]
    Call(AgentError),

    #[error("Failed to decode notify_create_canister response: {0:#}")]
    DecodeResponse(candid::Error),

    #[error("Failed to encode notify_create_canister arguments: {0:#}")]
    EncodeArguments(candid::Error),

    #[error("Failure reported by notify_create_canister: {0:?}")]
    Notify(NotifyError),
}
