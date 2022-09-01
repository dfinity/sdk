use crate::{
    commands::sns::config::SnsConfigOpts,
    lib::{environment::Environment, error::DfxResult},
};

use clap::Parser;

mod config;

/// SNS commands.
#[derive(Parser)]
#[clap(name("sns"))]
pub struct SnsOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(hide(true))]
    Config(SnsConfigOpts),
}

pub fn exec(env: &dyn Environment, cmd: SnsOpts) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Config(v) => config::exec(env, v),
    }
}
