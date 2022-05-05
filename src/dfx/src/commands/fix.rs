use clap::Parser;
use tokio::runtime::Runtime;

use crate::lib::{
    environment::Environment, error::DfxResult, migrate::migrate,
    provider::create_agent_environment,
};

#[derive(Parser)]
pub struct FixOpts {
    #[clap(long)]
    network: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: FixOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network)?;
    let runtime = Runtime::new().expect("Failed to create runtime");
    runtime.block_on(async { migrate(&env, env.get_network_descriptor().unwrap(), true).await })?;
    Ok(())
}
