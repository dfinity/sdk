use crate::lib::api_version::fetch_api_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};
use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::anyhow;
use clap::Clap;
use ic_types::principal::Principal as CanisterId;
use slog::info;
use tokio::runtime::Runtime;

/// Installs the wallet WASM to the provided canister id.
#[derive(Clap)]
pub struct DeployWalletOpts {
    /// The ID of the canister where the wallet WASM will be deployed.
    canister_id: String,
}

pub fn exec(env: &dyn Environment, opts: DeployWalletOpts, network: Option<String>) -> DfxResult {
    let agent_env = create_agent_environment(env, network.clone())?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    runtime.block_on(async { fetch_root_key_if_needed(&agent_env).await })?;

    let ic_api_version = runtime.block_on(async { fetch_api_version(&agent_env).await })?;
    if ic_api_version == "0.14.0" {
        info!(
            agent_env.get_logger(),
            "Unsupported replica api version '{}'", ic_api_version
        );
    } else {
        let identity_name = agent_env
            .get_selected_identity()
            .expect("No selected identity.")
            .to_string();
        let network = get_network_descriptor(&agent_env, network)?;

        let canister_id = opts.canister_id;
        match CanisterId::from_text(&canister_id) {
            Ok(id) => {
                runtime.block_on(async {
                    Identity::create_wallet(&agent_env, &network, &identity_name, Some(id)).await?;
                    DfxResult::Ok(())
                })?;
            }
            Err(err) => {
                anyhow!(format!(
                    "Cannot convert {} to a valid canister id. Candid error: {}",
                    canister_id, err
                ));
            }
        };
    }
    Ok(())
}
