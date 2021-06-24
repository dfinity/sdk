use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;

use ic_types::principal::Principal;

#[derive(Debug, PartialEq)]
pub enum CallSender {
    SelectedId,
    SelectedIdWallet(Principal),
    Wallet(Principal),
}

// Determine whether the selected Identity, the selected Identitys wallet,
// or the provided wallet canister ID should be the Sender of the call.
pub async fn call_sender(
    env: &dyn Environment,
    wallet: &Option<String>,
    no_wallet_flag: bool,
    should_wallet_proxy_by_default: bool,
) -> DfxResult<CallSender> {
    let sender = if wallet.is_none() && !no_wallet_flag {
        if should_wallet_proxy_by_default {
            let network = env
                .get_network_descriptor()
                .expect("No network descriptor.");
            let identity_name = env.get_selected_identity().expect("No selected identity.");
            let wallet =
                Identity::get_or_create_wallet_canister(env, network, &identity_name, true).await?;
            CallSender::SelectedIdWallet(*wallet.canister_id_())
        } else {
            CallSender::SelectedId
        }
    } else if let Some(id) = wallet {
        CallSender::Wallet(Principal::from_text(&id)?)
    } else if no_wallet_flag {
        CallSender::SelectedId
    } else {
        unreachable!()
    };
    Ok(sender)
}
