use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};

use anyhow::anyhow;
use clap::Clap;
use ic_types::Principal;
use ic_utils::call::SyncCall;
use ic_utils::interfaces::wallet::BalanceResult;
use slog::{error, info};
use tokio::runtime::Runtime;

/// Set the wallet for your identity on a network.
#[derive(Clap)]
pub struct SetWalletOpts {
    /// The Canister ID of the wallet to associate with this identity.
    #[clap(long)]
    canister_name: String,

    /// The network that the wallet exists on.
    #[clap(long)]
    network: Option<String>,

    /// Skip verification that the ID points to a correct wallet canister. Only useful for the local network.
    #[clap(long)]
    force: bool,
}

pub fn exec(env: &dyn Environment, opts: SetWalletOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.clone())?;
    let config = env.get_config_or_anyhow()?;
    let env = &agent_env;
    let log = env.get_logger();
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;

    let network = get_network_descriptor(&agent_env, opts.network.clone())?;

    let canister_name = opts.canister_name.as_str();
    let canister_id = match Principal::from_text(canister_name) {
        Ok(id) => id,
        Err(_) => {
            let canister_id = CanisterIdStore::for_env(env)?.get(canister_name)?;
            let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;
            canister_info.get_canister_id()?
        }
    };
    let force = opts.force;

    info!(
        log,
        "Setting wallet for identity '{}' on network '{}' to id '{}'",
        identity.name(),
        network.name,
        canister_id
    );

    identity.set_wallet_id(env, &network, canister_id)?;

    // Try to check the canister_id for a `wallet_balance()` if the network is local and available.
    // Otherwise we just trust the user.
    if network.is_local || force {
        let agent = env
            .get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

        let mut runtime = Runtime::new().expect("Unable to create a runtime");
        runtime
            .block_on(async {
                let _ = agent.status().await?;

                info!(
                    log,
                    "Checking availability of the canister on the network..."
                );

                let canister = identity.get_wallet(env, &network, false).await?;
                let balance = canister.wallet_balance().call().await;

                match balance {
                    Err(_) | Ok((BalanceResult { amount: 0 },)) => {
                        error!(
                            log,
                            "Impossible to read the canister. Make sure this is a valid wallet and the network is running. Use --force to skip this verification."
                        );
                        Err(anyhow!("Could not find the wallet or the wallet was invalid."))
                    }
                    _ => Ok(()),
                }
            })
            .map_err(DfxError::from)?;
    }

    Ok(())
}
