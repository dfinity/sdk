use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::provider::get_network_descriptor;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_types::Principal;
use ic_utils::call::SyncCall;
use slog::{error, info};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("create-wallet")
        .about(UserMessage::NewIdentity.to_str())
        .arg(
            Arg::with_name("canister-id")
                .help("The Canister ID of the wallet to associate with this identity.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("force")
                .help("Skip verification that the ID points to a correct wallet canister. Only useful for the local network.")
                .long("force"),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let log = env.get_logger();
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;

    let network = get_network_descriptor(env, args)?;
    let canister_id = Principal::from_text(args.value_of("canister-id").unwrap())?;
    let force = args.is_present("force");

    // Try to check the canister_id for a `cycle_balance()` if the network is local and available.
    // Otherwise we just trust the user.
    if network.is_local && force {
        let agent = env
            .get_agent()
            .ok_or(DfxError::CommandMustBeRunInAProject)?;

        let mut runtime = Runtime::new().expect("Unable to create a runtime");
        runtime
            .block_on(async {
                if agent.status().await.is_err() {
                    panic!("!!");
                }

                info!(
                    log,
                    "Checking availability of the canister on the network..."
                );

                identity.create_wallet(env, &network)?;
            })
            .map_err(DfxError::from)?;
    }
    info!(
        log,
        "Setting wallet for identity '{}' on network '{}' to id '{}'",
        identity.name(),
        network.name,
        canister_id
    );

    identity.set_wallet_id(env, &network, canister_id.clone())?;

    Ok(())
}
