use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;

use clap::Parser;
use tokio::runtime::Runtime;

mod install;

/// NNS commands.
#[derive(Parser)]
#[clap(name("nns"))]
pub struct NnsOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,

    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[clap(long, global(true))]
    network: Option<String>,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(hide(true))]
    Install(install::InstallOpts),
}

pub fn exec(env: &dyn Environment, opts: NnsOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network.clone())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Install(v) => install::exec(&env, v).await,
        }
    })
}
