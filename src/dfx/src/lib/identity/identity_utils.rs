use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use ic_types::principal::Principal;

#[derive(Debug, PartialEq)]
pub enum CallSender {
    SelectedId,
    Wallet(Principal),
}

// Determine whether the selected Identity
// or the provided wallet canister ID should be the Sender of the call.
pub async fn call_sender(_env: &dyn Environment, wallet: &Option<String>) -> DfxResult<CallSender> {
    let sender = if let Some(id) = wallet {
        CallSender::Wallet(Principal::from_text(&id)?)
    } else {
        CallSender::SelectedId
    };
    Ok(sender)
}
