use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

#[derive(Parser)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    name: String,
<<<<<<< HEAD
=======
    /// Use external (non-DFINITY) registry to install the extension.
    #[clap(long)]
    registry: Option<String>,
>>>>>>> 5d351902 (revert `install_as`-related functionality (tb introduced in another PR))
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    let spinner = env.new_spinner(format!("Installing extension: {}", opts.name).into());
<<<<<<< HEAD
    let mgr = env.new_extension_manager()?;
    mgr.install_extension(&opts.name)?;
=======
    mgr.install_extension(&opts.name, opts.registry.as_deref())?;
>>>>>>> 5d351902 (revert `install_as`-related functionality (tb introduced in another PR))
    spinner.finish_with_message(format!("Extension '{}' installed successfully", opts.name).into());
    Ok(())
}
