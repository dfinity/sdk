use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

#[derive(Parser)]
pub struct UninstallOpts {
    /// Specifies the name of the executable to uninstall.
    extension_name: String,
}

pub fn exec(env: &dyn Environment, opts: UninstallOpts) -> DfxResult<()> {
    let mut x = env.get_cache().get_extensions_directory().unwrap();
    x.push(opts.extension_name);
    std::fs::remove_dir_all(x).unwrap();
    Ok(())
}
