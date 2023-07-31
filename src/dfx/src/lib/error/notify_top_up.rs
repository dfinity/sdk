use crate::lib::ledger_types::NotifyError;
use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyTopUpError {
    #[error("Failed when calling notify_top_up: {0:#}")]
    Call(AgentError),

    #[error("Failed to decode notify_top_up response: {0:#}")]
    DecodeResponse(candid::Error),

    #[error("Failed to encode notify_top_up arguments: {0:#}")]
    EncodeArguments(candid::Error),

    #[error("Failure reported by notify_top_up: {0:?}")]
    Notify(NotifyError),
}
