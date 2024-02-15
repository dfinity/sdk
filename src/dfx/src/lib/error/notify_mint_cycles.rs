use crate::lib::ledger_types::NotifyError;
use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyMintCyclesError {
    #[error("Failed when calling notify_mint_cycles: {0:#}")]
    Call(AgentError),

    #[error("Failed to decode notify_mint_cycles response: {0:#}")]
    DecodeResponse(candid::Error),

    #[error("Failed to encode notify_mint_cycles arguments: {0:#}")]
    EncodeArguments(candid::Error),

    #[error("Failure reported by notify_mint_cycles: {0:?}")]
    Notify(NotifyError),
}
