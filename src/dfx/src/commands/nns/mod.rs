use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;
use crate::NetworkOpt;

use clap::Parser;
use tokio::runtime::Runtime;

mod import;
mod install;

/// NNS commands.
#[derive(Parser)]
#[clap(name("nns"))]
pub struct NnsOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,

    #[clap(flatten)]
    network: NetworkOpt,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(hide(true))]
    Import(import::ImportOpts),
    #[clap(hide(true))]
    Install(install::InstallOpts),
}

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
