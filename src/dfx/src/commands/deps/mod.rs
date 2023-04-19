use crate::lib::agent::create_agent_environment;
use crate::lib::{environment::Environment, error::DfxResult};
use crate::NetworkOpt;

use clap::Parser;
use tokio::runtime::Runtime;

mod deploy;
mod init;
mod pull;

/// Options for `dfx deps`.
#[derive(Parser)]
#[clap(name = "deps")]
pub struct DepsOpts {
    #[clap(flatten)]
    network: NetworkOpt,

    /// Arguments and flags for subcommands.
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Subcommands of `dfx deps`
#[derive(Parser)]
enum SubCommand {
    Pull(pull::DepsPullOpts),
    Init(init::DepsInitOpts),
    Deploy(deploy::DepsDeployOpts),
}

/// Executes `dfx deps` and its subcommands.
pub fn exec(env: &dyn Environment, opts: DepsOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Pull(v) => pull::exec(&agent_env, v).await,
            SubCommand::Init(v) => init::exec(&agent_env, v).await,
            SubCommand::Deploy(v) => deploy::exec(&agent_env, v).await,
        }
    })
}
