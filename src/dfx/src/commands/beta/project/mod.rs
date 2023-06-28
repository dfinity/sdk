use crate::lib::error::DfxResult;
use crate::Environment;

use clap::Parser;

mod import;

/// Project commands.
#[derive(Parser)]
#[command(name = "project")]
pub struct ProjectOpts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Import(import::ImportOpts),
}

pub fn exec(env: &dyn Environment, cmd: ProjectOpts) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Import(v) => import::exec(env, v),
    }
}
