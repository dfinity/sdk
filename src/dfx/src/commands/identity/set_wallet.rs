use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::provider::get_network_descriptor;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_types::Principal;
use slog::{error, info};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("set-wallet")
        .about(UserMessage::NewIdentity.to_str())
        .arg(
            Arg::with_name("canister-id")
                .help("The Canister ID of the wallet to associate with this identity.")
                .required(true)
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let log = env.get_logger();
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;

    let network = get_network_descriptor(env, args)?;
    let canister_id = Principal::from_text(args.value_of("canister-id").unwrap())?;

    // Try to check the canister_id for a `cycle_balance()` if the network is local and available.
    // Otherwise we just trust the user.
    if network.is_local {
        if let Some(agent) = env.get_agent() {
            let mut runtime = Runtime::new().expect("Unable to create a runtime");
            runtime.block_on(async {
                if agent.status().await.is_err() {
                    return Ok(());
                }

                info!(
                    log,
                    "Checking availability of canister on the local network..."
                );

                let canister = ic_utils::Canister::builder()
                    .with_agent(agent)
                    .with_canister_id(canister_id.clone())
                    .build()?;
                let balance = canister.query_("cycle_balance").call();

                if balance.is_err() || matches!(balance, Some(0)) {
                    error!(
                        log,
                        "Impossible to read the canister. Are you sure this is a valid wallet?"
                    )
                }

                Ok(())
            })?
        }
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
