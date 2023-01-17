use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::extension::manager::ExtensionsManager;

use clap::Parser;

use std::ffi::OsString;

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specifies the name of the extension to run.
    extension_name: OsString,
    /// Specifies the parameters to pass to the extension.
    params: Vec<OsString>,
}

impl From<Vec<OsString>> for RunOpts {
    fn from(params: Vec<OsString>) -> Self {
        let (extension_name, params) = params.split_first().unwrap();
        RunOpts {
            extension_name: extension_name.clone(),
            params: params.to_vec(),
        }
    }
}

pub fn exec(env: &dyn Environment, opts: RunOpts) -> DfxResult<()> {
    let mgr = ExtensionsManager::new(env).unwrap();
    mgr.run_extension(opts.extension_name, opts.params)
}
