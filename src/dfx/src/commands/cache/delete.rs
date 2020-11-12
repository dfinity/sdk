use crate::config::cache::delete_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Clap;

/// Deletes a specific versioned cache of dfx.
#[derive(Clap, Debug)]
#[clap(name("delete"))]
pub struct CacheDeleteOpts {
    #[clap(long)]
    version: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: CacheDeleteOpts) -> DfxResult {
    match opts.version {
        Some(v) => delete_version(v.as_str()).map(|_| {}),
        _ => env.get_cache().delete(),
    }
}
