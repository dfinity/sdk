use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use clap::Parser;
use dfx_core::config::cache::delete_version;

/// Deletes a specific versioned cache of dfx.
#[derive(Parser)]
#[command(name = "delete")]
pub struct CacheDeleteOpts {
    #[arg(long)]
    version: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: CacheDeleteOpts) -> DfxResult {
    match opts.version {
        Some(v) => delete_version(v.as_str()).map(|_| {}),
        _ => env.get_cache().delete(),
    }
    .map_err(DfxError::new)
}
