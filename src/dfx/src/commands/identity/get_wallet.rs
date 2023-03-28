use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use dfx_core::network::root_key::fetch_root_key_if_needed;

use crate::lib::identity::wallet::get_or_create_wallet;
use clap::Parser;
use tokio::runtime::Runtime;

/// Gets the canister ID for the wallet associated with your identity on a network.
#[derive(Parser)]
pub struct GetWalletOpts {}

pub fn exec(env: &dyn Environment, _opts: GetWalletOpts, network: Option<String>) -> DfxResult {
    let agent_env = create_agent_environment(env, network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    let agent = agent_env
        .get_agent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get HTTP client from environment."))?;
    let network = env.get_network_descriptor();
    runtime.block_on(async { fetch_root_key_if_needed(agent, network).await })?;

    let identity_name = agent_env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();
    let network = agent_env.get_network_descriptor();

    runtime.block_on(async {
        println!(
            "{}",
            get_or_create_wallet(&agent_env, network, &identity_name).await?
        );
        DfxResult::Ok(())
    })?;

    Ok(())
}
