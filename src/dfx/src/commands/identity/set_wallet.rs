use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::message::UserMessage;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_types::Principal;
use ic_utils::call::SyncCall;
use slog::{error, info};
use tokio::runtime::Runtime;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("set-wallet")
        .about(UserMessage::IdentitySetWallet.to_str())
        .arg(
            Arg::with_name("canister-id")
                .help("The Canister ID of the wallet to associate with this identity.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("network")
                .help("The network that the wallet exists on.")
                .long("network")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("force")
                .help("Skip verification that the ID points to a correct wallet canister. Only useful for the local network.")
                .long("force"),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let agent_env = create_agent_environment(env, args)?;
    let env = &agent_env;
    let log = env.get_logger();
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;

    let network = get_network_descriptor(env, args)?;
    let canister_id = Principal::from_text(args.value_of("canister-id").unwrap())?;
    let force = args.is_present("force");

    info!(
        log,
        "Setting wallet for identity '{}' on network '{}' to id '{}'",
        identity.name(),
        network.name,
        canister_id
    );

    identity.set_wallet_id(env, &network, canister_id)?;

    // Try to check the canister_id for a `cycle_balance()` if the network is local and available.
    // Otherwise we just trust the user.
    if network.is_local || force {
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

                let canister = identity.get_wallet(env, &network, false).await?;
                let balance = canister.cycle_balance().call().await;
                match balance {
                    Err(_) | Ok((0,)) => {
                        error!(
                            log,
                            "Impossible to read the canister. Make sure this is a valid wallet and the network is running. Use --force to skip this verification."
                        );
                        Err(DfxError::InvalidWalletCanister())
                    }
                    _ => Ok(()),
                }
            })
            .map_err(DfxError::from)?;
    }

    Ok(())
}
