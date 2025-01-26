use crate::lib::ledger_types::NotifyError;
use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyTopUpError {
    #[error("Failed when calling notify_top_up")]
    Call(#[source] AgentError),

    #[error("Failed to decode notify_top_up response")]
    DecodeResponse(#[source] candid::Error),

    #[error("Failed to encode notify_top_up arguments")]
    EncodeArguments(#[source] candid::Error),

    #[error("Failure reported by notify_top_up: {0:?}")]
    Notify(NotifyError),
}
