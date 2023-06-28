//! Code for the command line `dfx nns`.
#![warn(clippy::missing_docs_in_private_items)]
use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;
use tokio::runtime::Runtime;

mod import;
mod install;

/// Options for `dfx nns` and its subcommands.
#[derive(Parser)]
#[command(name = "nns")]
pub struct NnsOpts {
    /// `dfx nns` subcommand arguments.
    #[command(subcommand)]
    subcmd: SubCommand,
}

/// Command line options for subcommands of `dfx nns`.
#[derive(Parser)]
enum SubCommand {
    /// Import NNS API definitions and canister IDs.
    Import(import::ImportOpts),
    /// Install an NNS on the local dfx server.
    Install(install::InstallOpts),
}

/// Executes `dfx nns` and its subcommands.
pub fn exec(env: &dyn Environment, opts: NnsOpts) -> DfxResult {
    let env = create_agent_environment(env, None)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Import(v) => import::exec(&env, v).await,
            SubCommand::Install(v) => install::exec(&env, v).await,
        }
    })
}
