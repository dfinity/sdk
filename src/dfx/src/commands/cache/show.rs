use crate::config::cache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::{App, ArgMatches, Clap, IntoApp};

/// Shows the path of the cache used by this version.
#[derive(Clap)]
pub struct CacheShowOpts {}

pub fn construct() -> App<'static> {
    CacheShowOpts::into_app().name("show")
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches) -> DfxResult {
    let v = format!("{}", env.get_version());
    println!("{}", cache::get_bin_cache(&v)?.as_path().display());
    Ok(())
}
