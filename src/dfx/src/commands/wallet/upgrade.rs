use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::operations::canister::install_wallet;
use crate::lib::root_key::fetch_root_key_if_needed;
use anyhow::{anyhow, bail};
use clap::Parser;
use ic_agent::AgentError;
use ic_utils::interfaces::management_canister::builders::InstallMode;

/// Upgrade the wallet's Wasm module to the current Wasm bundled with DFX.
#[derive(Parser)]
pub struct UpgradeOpts {}

pub async fn exec(env: &dyn Environment, _opts: UpgradeOpts) -> DfxResult {
    let identity_name = env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();

    // Network descriptor will always be set.
    let network = env.get_network_descriptor();

    let canister_id =
        if let Some(principal) = Identity::wallet_canister_id(network, &identity_name)? {
            principal
        } else {
            bail!(
                "There is no wallet defined for identity '{}' on network '{}'.  Nothing to do.",
                identity_name,
                &network.name
            );
        };

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;
    match agent
        .read_state_canister_info(canister_id, "module_hash", false)
        .await
    {
        // If the canister is empty, this path does not exist.
        // The replica doesn't support negative lookups, therefore if the canister
        // is empty, the replica will return lookup_path([], Pruned _) = Unknown
        Err(AgentError::LookupPathUnknown(_)) | Err(AgentError::LookupPathAbsent(_)) => {
            bail!("The cycles wallet canister is empty. Try running `dfx identity deploy-wallet` to install code for the cycles wallet in this canister.")
        }
        Err(x) => bail!(x),
        _ => {}
    };

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    install_wallet(env, agent, canister_id, InstallMode::Upgrade).await?;

    println!("Upgraded the wallet wasm module.");
    Ok(())
}
