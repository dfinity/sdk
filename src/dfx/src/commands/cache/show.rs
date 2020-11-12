use crate::config::cache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Clap;

/// Shows the path of the cache used by this version.
#[derive(Clap)]
#[clap(name("show"))]
pub struct CacheShowOpts {}

pub fn exec(env: &dyn Environment) -> DfxResult {
    let v = format!("{}", env.get_version());
    println!("{}", cache::get_bin_cache(&v)?.as_path().display());
    Ok(())
}
