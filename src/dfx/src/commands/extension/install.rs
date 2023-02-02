use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

#[derive(Parser)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    name: String,
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    let spinner = env.new_spinner(format!("Installing extension: {}", opts.name).into());
    let mgr = env.new_extension_manager()?;
    mgr.install_extension(&opts.name)?;
    spinner.finish_with_message(format!("Extension '{}' installed successfully", opts.name).into());
    Ok(())
}
