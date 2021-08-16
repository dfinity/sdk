use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::assets::wallet_wasm;
use crate::util::expiry_duration;

use anyhow::{anyhow, bail};
use clap::Clap;
use ic_agent::AgentError;
use ic_utils::call::AsyncCall;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::ManagementCanister;

/// Upgrade the wallet's Wasm module to the current Wasm bundled with DFX.
#[derive(Clap)]
pub struct UpgradeOpts {}

pub async fn exec(env: &dyn Environment, _opts: UpgradeOpts) -> DfxResult {
    let identity_name = env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();

    // Network descriptor will always be set.
    let network = env.get_network_descriptor().unwrap();

    let canister_id = Identity::wallet_canister_id(env, network, &identity_name)?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;
    let install_mode = match agent
        .read_state_canister_info(canister_id, "module_hash")
        .await
    {
        Ok(_) => InstallMode::Upgrade,
        // If the canister is empty, this path does not exist.
        // The replica doesn't support negative lookups, therefore if the canister
        // is empty, the replica will return lookup_path([], Pruned _) = Unknown
        Err(AgentError::LookupPathUnknown(_)) | Err(AgentError::LookupPathAbsent(_)) => {
            bail!("The cycles wallet canister is empty. Try running `dfx identity deploy-wallet` to install code for the cycles wallet in this canister.")
        }
        Err(x) => bail!(x),
    };

    let wasm = wallet_wasm(env.get_logger())?;

    let mgr = ManagementCanister::create(
        env.get_agent()
            .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?,
    );

    mgr.install_code(&canister_id, wasm.as_slice())
        .with_mode(install_mode)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;

    let wallet = Identity::build_wallet_canister(canister_id, env)?;

    wallet
        .wallet_store_wallet_wasm(wasm)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;

    println!("Upgraded the wallet wasm module.");
    Ok(())
}
