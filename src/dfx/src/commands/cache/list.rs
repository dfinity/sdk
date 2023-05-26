use crate::config::dfx_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;
use dfx_core::config::cache::list_versions;
use std::io::Write;

/// Lists installed and used version.
#[derive(Parser)]
#[command(name = "list")]
pub struct CacheListOpts {}

pub fn exec(env: &dyn Environment, _opts: CacheListOpts) -> DfxResult {
    let mut current_printed = false;
    let current_version = env.get_version();
    let mut all_versions = list_versions()?;
    all_versions.sort();
    for version in all_versions {
        if current_version == &version {
            current_printed = true;
            // Same version, prefix with `*`.
            std::io::stderr().flush()?;
            print!("{}", version);
            std::io::stdout().flush()?;
            eprintln!(" *");
        } else {
            eprintln!("{}", version);
        }
    }

    if !current_printed {
        // The current version wasn't printed, so it's not in the cache.
        std::io::stderr().flush()?;
        print!("{}", dfx_version());
        std::io::stdout().flush()?;
        eprintln!(" [missing]");
    }

    Ok(())
}
