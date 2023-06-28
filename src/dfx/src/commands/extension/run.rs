use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;
use dfx_core::config::cache::get_bin_cache;

use std::ffi::OsString;

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specifies the name of the extension to run.
    name: OsString,
    /// Specifies the parameters to pass to the extension.
    params: Vec<OsString>,
}

impl From<Vec<OsString>> for RunOpts {
    fn from(value: Vec<OsString>) -> Self {
        let (extension_name, params) = value.split_first().unwrap();
        RunOpts {
            name: extension_name.clone(),
            params: params.to_vec(),
        }
    }
}

pub fn exec(env: &dyn Environment, opts: RunOpts) -> DfxResult<()> {
    let mgr = env.new_extension_manager()?;
    let dfx_version = &env.get_version().to_string();
    let path_to_dfx_cache = get_bin_cache(dfx_version)?;
    mgr.run_extension(path_to_dfx_cache, opts.name, opts.params)?;
    Ok(())
}
