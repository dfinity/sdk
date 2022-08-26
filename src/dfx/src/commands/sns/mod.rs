use crate::commands::sns::config::SnsConfigCommand;
use crate::{init_env, BaseOpts, DfxResult};

use crate::commands::sns::import::SnsImportOpts;
use clap::Parser;

mod config;
mod import;

/// SNS commands.
#[derive(Parser)]
#[clap(name("sns"))]
pub struct SnsCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(hide(true))]
    Config(SnsConfigCommand),
    #[clap(hide(true))]
    Import(BaseOpts<SnsImportOpts>),
}

pub fn dispatch(cmd: SnsCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Config(v) => config::dispatch(v),

        SubCommand::Import(v) => import::exec(&init_env(v.env_opts)?, v.command_opts),
    }
}
