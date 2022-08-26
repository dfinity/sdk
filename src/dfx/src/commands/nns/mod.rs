use crate::commands::NetworkOpts;
use crate::init_env;
use crate::lib::provider::create_agent_environment;
use crate::DfxResult;

use clap::Parser;
use tokio::runtime::Runtime;

mod import;
mod install;

/// NNS commands.
#[derive(Parser)]
#[clap(name("nns"))]
pub struct NnsCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(hide(true))]
    Import(NetworkOpts<import::ImportOpts>),

    #[clap(hide(true))]
    Install(NetworkOpts<install::InstallOpts>),
}

macro_rules! with_env {
    ($opts:expr, |$env:ident, $v:ident| $e:expr) => {{
        let NetworkOpts { base_opts, network } = $opts;
        let env = init_env(base_opts.env_opts)?;
        let $env = create_agent_environment(&env, network)?;
        let runtime = Runtime::new().expect("Unable to create a runtime");
        let $v = base_opts.command_opts;
        runtime.block_on($e)
    }};
}

pub fn dispatch(cmd: NnsCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Import(v) => with_env!(v, |env, v| import::exec(&env, v)),
        SubCommand::Install(v) => with_env!(v, |env, v| install::exec(&env, v)),
    }
}
