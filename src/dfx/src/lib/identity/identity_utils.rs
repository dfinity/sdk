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

#[allow(clippy::needless_lifetimes)]
pub async fn wallet_for_call_sender<'env>(
    env: &'env dyn Environment,
    call_sender: &CallSender,
    some_id: &Option<Principal>,
    create: bool,
) -> DfxResult<Canister<'env, Wallet>> {
    let network = env
        .get_network_descriptor()
        .expect("No network descriptor.");
    let identity_name = env.get_selected_identity().expect("No selected identity.");
    if call_sender == &CallSender::Wallet(some_id.clone()) {
        let id = some_id
            .as_ref()
            .expect("Wallet canister id should have been provided here.");
        Identity::build_wallet_canister(id.clone(), env)
    } else if call_sender == &CallSender::SelectedIdWallet(some_id.clone()) {
        Identity::get_or_create_wallet_canister(env, network, &identity_name, create).await
    } else {
        Err(anyhow!(
            "Attempted get a wallet for an invalid CallSender variant: {:?}",
            call_sender
        ))
    }
}
