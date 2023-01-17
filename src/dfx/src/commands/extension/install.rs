use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::extension::manager::ExtensionsManager;

use spinners::{Spinner, Spinners};
use clap::Parser;

#[derive(Parser)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    extension_name: String,
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    let mut sp = Spinner::new(Spinners::Dots9, format!("installing extension: {}", opts.extension_name).into());
    let mgr = ExtensionsManager::new(env).unwrap();
    let okk = mgr.install_extension(&opts.extension_name);
    sp.stop();
    okk
}
