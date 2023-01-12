use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

#[derive(Parser)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    extension_name: String,
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    let v = env.get_version();
    env.get_cache().install_extension(v, &opts.extension_name)
}
