use crate::DfxResult;

use crate::commands::sns::config::NnsConfigCommand;
use clap::Parser;

mod config;

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
    Config(NnsConfigCommand),
}

pub fn dispatch(cmd: SnsCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Config(v) => config::dispatch(v),
    }
}
