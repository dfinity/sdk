use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;

#[derive(Parser)]
pub struct UninstallOpts {
    /// Specifies the name of the executable to uninstall.
    name: String,
}

pub fn exec(env: &dyn Environment, opts: UninstallOpts) -> DfxResult<()> {
    let mgr = env.get_extension_manager();
    mgr.uninstall_extension(&opts.name)?;
    Ok(())
}
