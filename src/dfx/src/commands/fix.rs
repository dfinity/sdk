use clap::Parser;
use tokio::runtime::Runtime;

use crate::lib::{
    environment::Environment, error::DfxResult, migrate::migrate,
    provider::create_agent_environment,
};

/// Applies one-time fixes for known problems in the current environment caused by upgrading DFX.
/// Makes no changes that would not have been suggested by `dfx diagnose`.
#[derive(Parser)]
pub struct FixOpts {
    #[clap(long)]
    /// Override the network to connect to. By default the local network is used.
    network: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: FixOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network)?;
    let runtime = Runtime::new().expect("Failed to create runtime");
    runtime.block_on(async { migrate(&env, env.get_network_descriptor(), true).await })?;
    Ok(())
}
