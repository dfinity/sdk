use crate::lib::api_version::fetch_api_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};

use clap::Clap;
use slog::info;
use tokio::runtime::Runtime;

/// Get the canister ID of your wallet (or fail if there's no wallet) on a network.
#[derive(Clap)]
pub struct GetWalletOpts {}

pub fn exec(env: &dyn Environment, _opts: GetWalletOpts, network: Option<String>) -> DfxResult {
    let agent_env = create_agent_environment(env, network.clone())?;
    let log = agent_env.get_logger();

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let ic_api_version = runtime.block_on(async { fetch_api_version(&agent_env).await })?;

    if ic_api_version == "0.14.0" {
        info!(log, "Unsupported replica api version '{}'", ic_api_version);
    } else {
        let identity = IdentityManager::new(&agent_env)?.instantiate_selected_identity()?;
        let network = get_network_descriptor(&agent_env, network)?;

        runtime.block_on(async {
            println!(
                "{}",
                identity
                    .get_or_create_wallet(&agent_env, &network, true)
                    .await?
            );
            DfxResult::Ok(())
        })?;
    }
    Ok(())
}
