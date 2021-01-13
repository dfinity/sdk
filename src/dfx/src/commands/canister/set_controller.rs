use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::set_controller;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::expiry_duration;

use clap::Clap;
use ic_types::principal::Principal as CanisterId;

/// Sets the provided identity's name or its principal as the
/// new controller of a canister on the Internet Computer network.
#[derive(Clap)]
pub struct SetControllerOpts {
    /// Specifies the canister name or the canister identifier for the canister to be controlled.
    canister: String,

    /// Specifies the identity name or the principal of the new controller.
    new_controller: String,
}

pub async fn exec(env: &dyn Environment, opts: SetControllerOpts) -> DfxResult {
    let canister_id = match CanisterId::from_text(&opts.canister) {
        Ok(id) => id,
        Err(_) => CanisterIdStore::for_env(env)?.get(&opts.canister)?,
    };

    let controller_principal = match CanisterId::from_text(&opts.new_controller) {
        Ok(principal) => principal,
        Err(_) => {
            // If this is not a textual principal format, use the wallet of the person
            // and not its principal directly.
            let sender =
                IdentityManager::new(env)?.instantiate_identity_from_name(&opts.new_controller)?;
            let network = env.get_network_descriptor().expect("no network descriptor");
            sender.get_or_create_wallet(env, &network, true).await?
        }
    };

    let timeout = expiry_duration();
    fetch_root_key_if_needed(env).await?;

    set_controller(env, canister_id, controller_principal, timeout).await?;

    println!(
        "Set {:?} as controller of {:?}.",
        opts.new_controller, opts.canister
    );
    Ok(())
}
