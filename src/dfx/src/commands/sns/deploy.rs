//! Code for the command line `dfx sns deploy`.
use crate::lib::error::DfxResult;
use crate::{Environment, NetworkOpt};

use crate::lib::sns;
use crate::lib::sns::deploy::deploy_sns;
use clap::Parser;

/// Creates an SNS on a network.
///
/// # Arguments
/// - `env` - The execution environment, including the network to deploy to and connection credentials.
/// - `opts` - Deployment options.
#[derive(Parser)]
pub struct DeployOpts {
    #[clap(flatten)]
    network: NetworkOpt,
}

/// Executes the command line `dfx sns deploy`.
pub fn exec(env: &dyn Environment, opts: DeployOpts) -> DfxResult {
    println!("Creating SNS canisters.  This typically takes about one minute...");
    let config = env.get_config_or_anyhow()?;
    let path = config.get_project_root().join(sns::CONFIG_FILE_NAME);
    println!("{}", deploy_sns(env, &path, opts.network.network)?);
    Ok(())
}
