use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::extension::manager::ExtensionsManager;

use clap::Parser;

#[derive(Parser)]
pub struct UninstallOpts {
    /// Specifies the name of the executable to uninstall.
    extension_name: String,
}

pub fn exec(env: &dyn Environment, opts: UninstallOpts) -> DfxResult<()> {
    let mgr = ExtensionsManager::new(env)?;
    mgr.uninstall_extension(&opts.extension_name)?;
    Ok(())
}
