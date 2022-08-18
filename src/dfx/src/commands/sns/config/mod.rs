use crate::{BaseOpts, DfxResult};

use crate::init_env;
use clap::Parser;

mod create;
mod validate;

/// SNS config commands.
#[derive(Parser)]
#[clap(name("config"))]
pub struct NnsConfigCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Create(BaseOpts<create::CreateOpts>),
    Validate(BaseOpts<validate::ValidateOpts>),
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

pub fn dispatch(cmd: NnsConfigCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Create(v) => with_env!(v, |env, v| create::exec(&env, v)),
        SubCommand::Validate(v) => with_env!(v, |env, v| validate::exec(&env, v)),
    }
}
