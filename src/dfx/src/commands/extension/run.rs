use crate::lib::environment::Environment;
use crate::lib::error::{DfxResult, DfxError, ExtensionError};
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

impl TryFrom<Vec<OsString>> for RunOpts {
    type Error = DfxError;

    fn try_from(value: Vec<OsString>) -> Result<Self, Self::Error> {
        let (extension_name, params) = value.split_first().ok_or(DfxError::new(
            ExtensionError::ExtensionError("hard to imagine what went wrong here".to_string())
        ))?;
        Ok(RunOpts {
            extension_name: extension_name.clone(),
            params: params.to_vec(),
        })
    }
}

pub fn exec(env: &dyn Environment, opts: RunOpts) -> DfxResult<()> {
    let mgr = ExtensionsManager::new(env)?;
    mgr.run_extension(opts.extension_name, opts.params)
}
