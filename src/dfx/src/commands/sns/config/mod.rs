use crate::{BaseOpts, DfxResult};

use crate::init_env;
use clap::Parser;

mod create;
mod validate;

#[derive(Parser)]
pub struct ConfigOpts {}

/// SNS config commands.
#[derive(Parser)]
#[clap(name("config"))]
pub struct SnsConfigCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Create(BaseOpts<create::CreateOpts>),
    Validate(BaseOpts<validate::ValidateOpts>),
}

pub fn dispatch(cmd: SnsConfigCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Create(v) => create::exec(&init_env(v.env_opts)?, v.command_opts),
        SubCommand::Validate(v) => validate::exec(&init_env(v.env_opts)?, v.command_opts),
    }
}
