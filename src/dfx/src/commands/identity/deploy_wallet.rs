use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::bail;
use candid::Principal as CanisterId;
use clap::Parser;
use tokio::runtime::Runtime;

/// Installs the wallet WASM to the provided canister id.
#[derive(Parser)]
pub struct DeployWalletOpts {
    /// The ID of the canister where the wallet WASM will be deployed.
    canister_id: String,
}

pub fn exec(env: &dyn Environment, opts: DeployWalletOpts, network: Option<String>) -> DfxResult {
    let agent_env = create_agent_environment(env, network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    runtime.block_on(async { fetch_root_key_if_needed(&agent_env).await })?;

    let identity_name = agent_env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();
    let network = agent_env.get_network_descriptor();

    let canister_id = opts.canister_id;
    match CanisterId::from_text(&canister_id) {
        Ok(id) => {
            runtime.block_on(async {
                Identity::create_wallet(&agent_env, network, &identity_name, Some(id)).await?;
                DfxResult::Ok(())
            })?;
        }
        Err(err) => {
            bail!(
                "Cannot convert {} to a valid canister id. Candid error: {}",
                canister_id,
                err
            );
        }
    };
    Ok(())
}
