use crate::config::cache::VersionCache;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use clap::Parser;

/// Forces unpacking the cache from this dfx version.
#[derive(Parser)]
#[command(name = "install")]
pub struct CacheInstall {}

pub fn exec(env: &dyn Environment, _opts: CacheInstall) -> DfxResult {
    VersionCache::force_install(env, &env.get_cache().version_str()).map_err(DfxError::from)?;
    Ok(())
}
