use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_agent::ManagementCanister;
use ic_types::principal::Principal as CanisterId;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("set-controller")
        .about(UserMessage::CallCanister.to_str())
        .arg(
            Arg::with_name("canister_id")
                .takes_value(true)
                .help(UserMessage::CreateCanisterName.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name("new_controller")
                .takes_value(true)
                .help(UserMessage::CanisterName.to_str())
                .required(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let canister_id = args.value_of("canister_id").unwrap();
    let new_controller = args.value_of("new_controller").unwrap();
    let timeout = expiry_duration();

    let mgr = ManagementCanister::new(
        env.get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?,
    );

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(mgr.set_controller(
        waiter_with_timeout(timeout),
        &CanisterId::from_text(canister_id)?,
        &CanisterId::from_text(new_controller)?,
    ))?;

    Ok(())
}
