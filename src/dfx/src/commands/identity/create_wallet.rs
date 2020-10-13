use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};
use clap::{App, Arg, ArgMatches, SubCommand};
use slog::info;
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("create-wallet")
        .about(UserMessage::IdentityCreateWallet.to_str())
        .arg(
            Arg::with_name("network")
                .help("The network that the wallet exists on.")
                .long("network")
                .takes_value(true),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let env = create_agent_environment(env, args)?;
    let log = env.get_logger();
    let identity = IdentityManager::new(&env)?.instantiate_selected_identity()?;
    let network = get_network_descriptor(&env, args)?;

    // Try to check the canister_id for a `cycle_balance()` if the network is local and available.
    // Otherwise we just trust the user.
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

            let canister_id = identity.create_wallet(&env, &network).await?;
            info!(
                log,
                "Created wallet for identity '{}' on network '{}' with id '{}'",
                identity.name(),
                network.name,
                canister_id
            );

            identity.set_wallet_id(&env, &network, canister_id)?;

            DfxResult::Ok(())
        })
        .map_err(DfxError::from)?;

    Ok(())
}
