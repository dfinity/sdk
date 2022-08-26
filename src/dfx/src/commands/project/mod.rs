use crate::{init_env, BaseOpts, DfxResult};

use clap::Parser;

mod import;

/// Ledger commands.
#[derive(Parser)]
#[clap(name("project"))]
pub struct ProjectCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Import(BaseOpts<import::ImportOpts>),
}

pub fn dispatch(cmd: ProjectCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Import(v) => import::exec(&init_env(v.env_opts)?, v.command_opts),
    }
}
