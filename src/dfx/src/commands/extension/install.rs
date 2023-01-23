use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;
use spinners::{Spinner, Spinners};

#[derive(Parser)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    name: String,
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    let mut sp = Spinner::new(
        Spinners::Dots9,
        format!("Installing extension: {}", opts.name),
    );
    let mgr = env.new_extension_manager()?;
    mgr.install_extension(&opts.name)?;
    sp.stop();
    Ok(())
}
