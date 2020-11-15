use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::{App, ArgMatches, Clap, IntoApp};

/// Forces unpacking the cache from this dfx version.
#[derive(Clap)]
#[clap(name("install"))]
pub struct CacheInstall {}

pub fn construct() -> App<'static> {
    CacheInstall::into_app()
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches) -> DfxResult {
    env.get_cache().force_install()
}
