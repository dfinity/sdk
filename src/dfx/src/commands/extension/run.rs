use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;


use std::ffi::OsString;

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specifies the name of the extension to run.
    pub name: OsString,
    /// Specifies the parameters to pass to the extension.
    pub params: Vec<OsString>,
}

impl From<Vec<OsString>> for RunOpts {
    fn from(value: Vec<OsString>) -> Self {
        dbg!(&value);
        let (extension_name, params) = value.split_first().unwrap();
        RunOpts {
            name: extension_name.clone(),
            params: params.to_vec(),
        }
    }
}

pub fn exec(env: &dyn Environment, opts: RunOpts) -> DfxResult<()> {
    let mgr = env.new_extension_manager()?;
    let _dfx_version = &env.get_version().to_string();
    mgr.run_extension(opts.name, opts.params)?;
    Ok(())
}
