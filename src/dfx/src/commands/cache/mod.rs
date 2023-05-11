use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

mod delete;
mod install;
mod list;
mod show;

/// Manages the dfx version cache.
#[derive(Parser)]
#[command(name = "cache")]
pub struct CacheOpts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    Delete(delete::CacheDeleteOpts),
    Install(install::CacheInstall),
    List(list::CacheListOpts),
    Show(show::CacheShowOpts),
}

pub fn exec(env: &dyn Environment, opts: CacheOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::Delete(v) => delete::exec(env, v),
        SubCommand::Install(v) => install::exec(env, v),
        SubCommand::List(v) => list::exec(env, v),
        SubCommand::Show(v) => show::exec(env, v),
    }
}
