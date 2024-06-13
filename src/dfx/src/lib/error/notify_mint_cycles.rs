use crate::lib::ledger_types::NotifyError;
use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyMintCyclesError {
    #[error("Failed when calling notify_mint_cycles")]
    Call(#[source] AgentError),

    #[error("Failed to decode notify_mint_cycles response")]
    DecodeResponse(#[source] candid::Error),

    #[error("Failed to encode notify_mint_cycles arguments")]
    EncodeArguments(#[source] candid::Error),

    #[error("Failure reported by notify_mint_cycles: {0:?}")]
    Notify(NotifyError),
}
