use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};
use crate::lib::root_key::fetch_root_key_if_needed;

use anyhow::Context;
use clap::Parser;
use tokio::runtime::Runtime;

/// Gets the canister ID for the wallet associated with your identity on a network.
#[derive(Parser)]
pub struct GetWalletOpts {}

pub fn exec(env: &dyn Environment, _opts: GetWalletOpts, network: Option<String>) -> DfxResult {
    let agent_env = create_agent_environment(env, network.clone())
        .context("Failed to create AgentEnvironment.")?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    runtime
        .block_on(async { fetch_root_key_if_needed(&agent_env).await })
        .context("Failed to fetch root key.")?;

    let identity_name = agent_env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();
    let network = get_network_descriptor(&agent_env, network)
        .context("Failed to determine network descriptor.")?;

    runtime.block_on(async {
        println!(
            "{}",
            Identity::get_or_create_wallet(&agent_env, &network, &identity_name, false)
                .await
                .with_context(|| format!(
                    "Failed to get or create wallet for {}.",
                    &identity_name
                ))?
        );
        DfxResult::Ok(())
    })?;

    Ok(())
}
