use crate::config::cache::delete_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};

/// Deletes a specific versioned cache of dfx.
#[derive(Clap)]
#[clap(name("delete"))]
pub struct CacheDeleteOpts {
    #[clap(long)]
    version: Option<String>,
}

pub fn construct() -> App<'static> {
    CacheDeleteOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: CacheDeleteOpts = CacheDeleteOpts::from_arg_matches(args);
    match opts.version {
        Some(v) => delete_version(v.as_str()).map(|_| {}),
        _ => env.get_cache().delete(),
    }
}
