use crate::config::cache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::Context;
use clap::Parser;

/// Shows the path of the cache used by this version.
#[derive(Parser)]
#[clap(name("show"))]
pub struct CacheShowOpts {}

pub fn exec(env: &dyn Environment, _opts: CacheShowOpts) -> DfxResult {
    let v = format!("{}", env.get_version());
    println!(
        "{}",
        cache::get_bin_cache(&v)
            .context(format!("Failed to get binary cache for version {}", &v))?
            .as_path()
            .display()
    );
    Ok(())
}
