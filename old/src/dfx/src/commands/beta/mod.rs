use crate::lib::error::DfxResult;
use crate::Environment;
use clap::Parser;

mod project;

/// Beta commands.
#[derive(Parser)]
#[command(name = "beta")]
pub struct BetaOpts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Project(project::ProjectOpts),
}

pub fn exec(env: &dyn Environment, cmd: BetaOpts) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Project(v) => project::exec(env, v),
    }
}
