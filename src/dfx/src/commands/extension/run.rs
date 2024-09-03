use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use std::ffi::OsString;

#[derive(Parser, Debug)]
pub struct RunOpts {
    /// Specifies the name of the extension to run.
    name: OsString,
    /// Specifies the parameters to pass to the extension.
    #[arg(allow_hyphen_values = true)]
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
    let mgr = env.get_extension_manager();
    let project_root = env
        .get_config()?
        .map(|c| c.get_project_root().to_path_buf());
    mgr.run_extension(opts.name, opts.params, project_root)?;
    Ok(())
}
