use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};
use crate::lib::root_key::fetch_root_key_if_needed;

use clap::Clap;
use tokio::runtime::Runtime;

/// Gets the canister ID for the wallet associated with your identity on a network.
#[derive(Clap)]
pub struct GetWalletOpts {}

pub fn exec(env: &dyn Environment, _opts: GetWalletOpts, network: Option<String>) -> DfxResult {
    let agent_env = create_agent_environment(env, network.clone())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    runtime.block_on(async { fetch_root_key_if_needed(&agent_env).await })?;

    let identity_name = agent_env
        .get_selected_identity()
        .expect("No selected identity.")
        .to_string();
    let network = get_network_descriptor(&agent_env, network)?;

    runtime.block_on(async {
        println!(
            "{}",
            Identity::get_or_create_wallet(&agent_env, &network, &identity_name, true).await?
        );
        DfxResult::Ok(())
    })?;

    Ok(())
}
