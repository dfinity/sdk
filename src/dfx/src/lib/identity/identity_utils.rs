use crate::lib::error::DfxResult;

use anyhow::Context;
use candid::Principal;
use fn_error_context::context;

#[derive(Debug, PartialEq, Eq)]
pub enum CallSender {
    SelectedId,
    Wallet(Principal),
}

// Determine whether the selected Identity
// or the provided wallet canister ID should be the Sender of the call.
#[context("Failed to determine call sender.")]
pub async fn call_sender(wallet: &Option<String>) -> DfxResult<CallSender> {
    let sender = if let Some(id) = wallet {
        CallSender::Wallet(
            Principal::from_text(id)
                .with_context(|| format!("Failed to read principal from {:?}.", id))?,
        )
    } else {
        CallSender::SelectedId
    };
    Ok(sender)
}
