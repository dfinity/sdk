use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Clap;

/// Forces unpacking the cache from this dfx version.
#[derive(Clap)]
#[clap(name("install"))]
pub struct CacheInstall {}

pub fn exec(env: &dyn Environment) -> DfxResult {
    env.get_cache().force_install()
}
