use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::toolchain::Toolchain;

use clap::Clap;

/// Install or update given toolchain(s)
#[derive(Clap)]
#[clap(name("install"))]
pub struct ToolchainInstall {
    /// Toolchain name, such as '0.6.22', '0.6', 'latest'
    #[clap(required = true, min_values = 1)]
    toolchains: Vec<String>,
}

pub fn exec(_env: &dyn Environment, opts: ToolchainInstall) -> DfxResult {
    for s in opts.toolchains {
        let toolchain = s.parse::<Toolchain>()?;
        toolchain.update()?;
    }
    Ok(())
}
