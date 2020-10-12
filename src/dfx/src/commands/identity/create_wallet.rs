use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::provider::get_network_descriptor;
use clap::{App, ArgMatches, SubCommand};
use ic_types::Principal;
use slog::info;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("create-wallet").about(UserMessage::IdentityCreateWallet.to_str())
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
                let _ = agent.status().await?;

                info!(
                    log,
                    "Checking availability of the canister on the network..."
                );

                identity.create_wallet(env, &network)?;
                DfxResult::Ok(())
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
