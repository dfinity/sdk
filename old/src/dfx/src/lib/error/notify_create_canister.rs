use crate::lib::ledger_types::NotifyError;
use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyCreateCanisterError {
    #[error("Failed when calling notify_create_canister")]
    Call(#[source] AgentError),

    #[error("Failed to decode notify_create_canister response")]
    DecodeResponse(#[source] candid::Error),

    #[error("Failed to encode notify_create_canister arguments")]
    EncodeArguments(#[source] candid::Error),

    #[error("Failure reported by notify_create_canister: {0:?}")]
    Notify(NotifyError),
}
