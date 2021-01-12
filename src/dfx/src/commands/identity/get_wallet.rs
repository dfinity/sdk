use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::provider::{create_agent_environment, get_network_descriptor};
use clap::Clap;
use tokio::runtime::Runtime;

/// Get the canister ID of your wallet (or fail if there's no wallet) on a network.
#[derive(Clap)]
pub struct GetWalletOpts {
    /// The network that the wallet exists on.
    #[clap(long)]
    network: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: GetWalletOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.clone())?;
    // let agent_env = create_agent_environment(env, args)?;
    let identity = IdentityManager::new(&agent_env)?.instantiate_selected_identity()?;
    let network = get_network_descriptor(&agent_env, opts.network)?;

    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        println!(
            "{}",
            identity
                .get_or_create_wallet(&agent_env, &network, true)
                .await?
        );
        DfxResult::Ok(())
    })?;

    Ok(())
}
