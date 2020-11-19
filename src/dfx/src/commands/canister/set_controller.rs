use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;
use clap::Clap;
use ic_agent::Identity;
use ic_types::principal::Principal as CanisterId;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use tokio::runtime::Runtime;

/// Sets the provided identity's name or its principal as the
/// new controller of a canister on the Internet Computer network.
#[derive(Clap)]
#[clap(name("set-controller"))]
pub struct SetControllerOpts {
    /// Specifies the canister name or the canister identifier for the canister to be controlled.
    canister: String,

    /// Specifies the identity name or the principal of the new controller.
    new_controller: String,
}

pub fn exec(env: &dyn Environment, opts: SetControllerOpts) -> DfxResult {
    let canister_id = match CanisterId::from_text(&opts.canister) {
        Ok(id) => id,
        Err(_) => CanisterIdStore::for_env(env)?.get(&opts.canister)?,
    };

    let controller_principal = match CanisterId::from_text(&opts.new_controller) {
        Ok(principal) => principal,
        Err(_) => IdentityManager::new(env)?
            .instantiate_identity_from_name(&opts.new_controller)?
            .sender()?,
    };

    let timeout = expiry_duration();

    let mgr = ManagementCanister::create(
        env.get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?,
    );

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(
        mgr.set_controller(&canister_id, &controller_principal)
            .call_and_wait(waiter_with_timeout(timeout)),
    )?;

    println!(
        "Set {:?} as controller of {:?}.",
        opts.new_controller, opts.canister
    );
    Ok(())
}
