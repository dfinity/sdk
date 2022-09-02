use crate::{
    commands::sns::config::SnsConfigOpts,
    commands::sns::import::SnsImportOpts,
    lib::{environment::Environment, error::DfxResult},
};

use clap::Parser;

mod config;
mod import;

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
    #[clap(hide(true))]
    Import(SnsImportOpts),
}

pub fn exec(env: &dyn Environment, cmd: SnsOpts) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Config(v) => config::exec(env, v),
        SubCommand::Import(v) => import::exec(env, v),
    }
}
