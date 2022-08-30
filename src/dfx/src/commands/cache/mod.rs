use crate::lib::error::DfxResult;
use crate::{init_env, BaseOpts};

use clap::Parser;

mod delete;
mod install;
mod list;
mod show;

/// Manages the dfx version cache.
#[derive(Parser)]
#[clap(name("cache"))]
pub struct CacheCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    Delete(BaseOpts<delete::CacheDeleteOpts>),
    Install(BaseOpts<install::CacheInstall>),
    List(BaseOpts<list::CacheListOpts>),
    Show(BaseOpts<show::CacheShowOpts>),
}

pub fn dispatch(cmd: CacheCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Delete(v) => delete::exec(&init_env(v.env_opts)?, v.command_opts),
        SubCommand::Install(v) => install::exec(&init_env(v.env_opts)?, v.command_opts),
        SubCommand::List(v) => list::exec(&init_env(v.env_opts)?, v.command_opts),
        SubCommand::Show(v) => show::exec(&init_env(v.env_opts)?, v.command_opts),
    }
}
