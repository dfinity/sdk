use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

mod delete;
mod install;
mod list;
mod show;

/// Manages the dfx version cache.
#[derive(Clap)]
#[clap(name("cache"))]
pub struct CacheOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Delete(delete::CacheDeleteOpts),
    Install(install::CacheInstall),
    List(list::CacheListOpts),
    Show(show::CacheShowOpts),
}

pub fn construct() -> App<'static> {
    CacheOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CacheOpts = CacheOpts::from_arg_matches(args);
    match opts.subcmd {
        SubCommand::Delete(v) => delete::exec(env, v),
        SubCommand::Install(_v) => install::exec(env),
        SubCommand::List(_v) => list::exec(env),
        SubCommand::Show(_v) => show::exec(env),
    }
}
