use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;

use ic_types::principal::Principal;

#[derive(Debug, PartialEq)]
pub enum CallSender {
    SelectedId,
    SelectedIdWallet(Option<Principal>),
    Wallet(Option<Principal>),
}

// Determine whether the selected Identity, the selected Identitys wallet,
// or the provided wallet canister ID should be the Sender of the call.
pub async fn call_sender(
    env: &dyn Environment,
    wallet: &Option<String>,
    no_wallet: bool,
) -> DfxResult<CallSender> {
    let sender = if wallet.is_none() && !no_wallet {
        let network = env
            .get_network_descriptor()
            .expect("No network descriptor.");
        let identity_name = env.get_selected_identity().expect("No selected identity.");
        let some_id = match Identity::wallet_canister_id(env, network, &identity_name) {
            Ok(id) => Some(id),
            Err(_) => None,
        };
        CallSender::SelectedIdWallet(some_id)
    } else if let Some(id) = wallet {
        let id = Principal::from_text(&id)?;
        CallSender::Wallet(Some(id))
    } else if no_wallet {
        CallSender::SelectedId
    } else {
        unreachable!()
    };
    Ok(sender)
}
