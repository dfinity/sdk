use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::extension::manager::ExtensionsManager;

use clap::Parser;
// use spinners::{Spinner, Spinners};

#[derive(Parser)]
pub struct InfoOpts {
    /// Specifies the name of the extension to install.
    extension_name: String,
}

pub fn exec(env: &dyn Environment, opts: InfoOpts) -> DfxResult<()> {
    let mgr = ExtensionsManager::new(env).unwrap();
    let md = mgr.get_extension_metadata(&opts.extension_name)?;
    println!("{}", md);
    Ok(())
}
