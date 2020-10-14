use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;
use candid::CandidType;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::Identity;
use ic_types::principal::Principal as CanisterId;
use ic_types::Principal;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::ManagementCanister;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("set-controller")
        .about(UserMessage::SetController.to_str())
        .arg(
            Arg::with_name("canister")
                .takes_value(true)
                .help(UserMessage::SetControllerCanister.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name("new-controller")
                .takes_value(true)
                .help(UserMessage::NewController.to_str())
                .required(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let canister = args.value_of("canister").unwrap();
    let canister_id = match CanisterId::from_text(canister) {
        Ok(id) => id,
        Err(_) => CanisterIdStore::for_env(env)?.get(canister)?,
    };

    let new_controller = args.value_of("new-controller").unwrap();
    let controller_principal = match CanisterId::from_text(new_controller) {
        Ok(principal) => principal,
        Err(_) => IdentityManager::new(env)?
            .instantiate_identity_from_name(new_controller)?
            .sender()?,
    };
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let network = env.get_network_descriptor().expect("no network descriptor");

    let timeout = expiry_duration();

    let mgr = ManagementCanister::create(
        env.get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?,
    );

    #[derive(CandidType)]
    struct Argument {
        canister_id: Principal,
        new_controller: Principal,
    }

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        let wallet = identity.get_wallet(env, &network, false).await?;

        wallet
            .call_forward(
                mgr.update_("set_controller")
                    .with_arg(Argument {
                        canister_id,
                        new_controller: controller_principal.clone(),
                    })
                    .build(),
                0,
            )?
            .call_and_wait(waiter_with_timeout(timeout))
            .await?;

        DfxResult::Ok(())
    })?;

    println!(
        "Set {} as controller of {:?}.",
        controller_principal, canister
    );
    Ok(())
}
