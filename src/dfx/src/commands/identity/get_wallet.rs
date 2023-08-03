use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::wallet::get_or_create_wallet;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::NetworkOpt;
use clap::Parser;
use tokio::runtime::Runtime;

/// Gets the canister ID for the wallet associated with your identity on a network.
#[derive(Parser)]
pub struct GetWalletOpts {}

pub fn exec(env: &dyn Environment, _opts: GetWalletOpts, network: NetworkOpt) -> DfxResult {
    let agent_env = create_agent_environment(env, network.to_network_name())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    runtime.block_on(async { fetch_root_key_if_needed(&agent_env).await })?;

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
