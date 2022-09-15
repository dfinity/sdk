//! Code for the command line `dfx nns`.
#![warn(clippy::missing_docs_in_private_items)]
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;
use crate::NetworkOpt;

use clap::Parser;
use tokio::runtime::Runtime;

mod import;
mod install;

/// Options for `dfx nns` and its subcommands.
#[derive(Parser)]
#[clap(name("nns"))]
pub struct NnsOpts {
    /// `dfx nns` subcommand arguments.
    #[clap(subcommand)]
    subcmd: SubCommand,

    /// An argument to choose the network from those specified in dfx.json.
    #[clap(flatten)]
    network: NetworkOpt,
}

/// Command line options for subcommands of `dfx nns`.
#[derive(Parser)]
enum SubCommand {
    /// Options for importing NNS API definitions and canister IDs.
    #[clap(hide(true))]
    Import(import::ImportOpts),
    /// Options for installing an NNS.
    #[clap(hide(true))]
    Install(install::InstallOpts),
}

/// Executes `dfx nns` and its subcommands.
pub fn exec(env: &dyn Environment, opts: NnsOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network.network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Import(v) => import::exec(&env, v).await,
            SubCommand::Install(v) => install::exec(&env, v).await,
        }
    })
}
