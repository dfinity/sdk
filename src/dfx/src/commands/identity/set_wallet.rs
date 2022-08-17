use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::create_agent_environment;

use anyhow::{anyhow, Context};
use candid::Principal;
use clap::Parser;
use ic_utils::interfaces::wallet::BalanceResult;
use slog::{error, info};
use tokio::runtime::Runtime;

/// Sets the wallet canister ID to use for your identity on a network.
#[derive(Parser)]
pub struct SetWalletOpts {
    /// The Canister ID of the wallet to associate with this identity.
    canister_name: String,

    /// Skip verification that the ID points to a correct wallet canister. Only useful for the local network.
    #[clap(long)]
    force: bool,
}

pub fn exec(env: &dyn Environment, opts: SetWalletOpts, network: Option<String>) -> DfxResult {
    let agent_env = create_agent_environment(env, network)?;
    let env = &agent_env;
    let log = env.get_logger();

    let runtime = Runtime::new().expect("Unable to create a runtime");

    let identity_name = agent_env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();

    let network = agent_env.get_network_descriptor();

    let canister_name = opts.canister_name.as_str();
    let canister_id = match Principal::from_text(canister_name) {
        Ok(id) => id,
        Err(_) => {
            let config = env.get_config_or_anyhow()?;
            let canister_id = CanisterIdStore::for_env(env)?.get(canister_name)?;
            let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;
            canister_info.get_canister_id()?
        }
    };
    let force = opts.force;

    // Try to check the canister_id for a `wallet_balance()` if the network is not the IC and available.
    // Otherwise we just trust the user.
    if force {
        info!(
            log,
            "Skipping verification of availability of the canister on the network due to --force..."
        );
    } else {
        let agent = env
            .get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

        runtime
            .block_on(async {
                let _ = agent.status().await.context("Failed to read network status.")?;

                info!(
                    log,
                    "Checking availability of the canister on the network..."
                );

                let canister = Identity::build_wallet_canister(canister_id, env).await?;
                let balance = canister.wallet_balance().await;

                match balance {
                    Ok(BalanceResult { amount: 0 }) => {
                        error!(
                            log,
                            "Impossible to read the canister. Make sure this is a valid wallet{}. Use --force to skip this verification.",
                            if !network.is_ic { " and the network is running" } else { "" },
                        );
                        Err(anyhow!("Could not find the wallet or the wallet was invalid."))
                    },
                    Err(err) => {
                        Err(anyhow!("Unable to access the wallet: {}", err))
                    },
                    _ => {
                        Ok(())
                    },
                }
            })
            .map_err(DfxError::from)?;
    }

    info!(
        log,
        "Setting wallet for identity '{}' on network '{}' to id '{}'",
        identity_name,
        network.name,
        canister_id
    );
    Identity::set_wallet_id(network, &identity_name, canister_id)?;
    info!(log, "Wallet set successfully.");

    Ok(())
}
