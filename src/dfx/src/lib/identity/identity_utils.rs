use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;

use anyhow::anyhow;
use ic_types::principal::Principal;
use ic_utils::canister::Canister;
use ic_utils::interfaces::Wallet;

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
    no_wallet: bool,
) -> DfxResult<CallSender> {
    let sender = if wallet.is_none() && !no_wallet {
        let network = env
            .get_network_descriptor()
            .expect("No network descriptor.");
        let identity_name = env.get_selected_identity().expect("No selected identity.");
        let wallet =
            Identity::get_or_create_wallet_canister(env, network, &identity_name, true).await?;
        CallSender::SelectedIdWallet(wallet.canister_id_().clone())
    } else if let Some(id) = wallet {
        CallSender::Wallet(Principal::from_text(&id)?)
    } else if no_wallet {
        CallSender::SelectedId
    } else {
        unreachable!()
    };
    Ok(sender)
}

#[allow(clippy::needless_lifetimes)]
pub async fn wallet_for_call_sender<'env>(
    env: &'env dyn Environment,
    call_sender: &CallSender,
    wallet_id: &Principal,
) -> DfxResult<Canister<'env, Wallet>> {
    if call_sender == &CallSender::Wallet(wallet_id.clone())
        || call_sender == &CallSender::SelectedIdWallet(wallet_id.clone())
    {
        Identity::build_wallet_canister(wallet_id.clone(), env)
    } else {
        Err(anyhow!(
            "Attempted to get a wallet for an invalid CallSender variant: {:?}",
            call_sender
        ))
    }
}
