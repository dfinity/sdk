//! Code for the command line `dfx sns deploy`.
use crate::lib::error::DfxResult;
use crate::Environment;

use crate::lib::sns;
use crate::lib::sns::deploy::deploy_sns;
use clap::Parser;

/// Creates an SNS on a network.
///
/// # Arguments
/// - `env` - The execution environment, including the network to deploy to and connection credentials.
/// - `opts` - Deployment options.
#[derive(Parser)]
pub struct DeployOpts {}

/// Executes the command line `dfx sns deploy`.
pub fn exec(env: &dyn Environment, _opts: DeployOpts) -> DfxResult {
    println!("Creating SNS canisters.  This typically takes about one minute...");
    let config = env.get_config_or_anyhow()?;
    let path = config.get_project_root().join(sns::CONFIG_FILE_NAME);
    let cache = env.get_cache();

    println!("{}", deploy_sns(cache.as_ref(), &path)?);
    Ok(())
}
