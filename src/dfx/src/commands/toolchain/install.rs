use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::toolchain::ToolchainDesc;

use clap::Clap;

#[derive(Clap)]
#[clap(name("install"))]
pub struct ToolchainInstall {
    /// Toolchain name, such as '0.6.22', '0.6', 'latest'
    #[clap(required = true, min_values = 1)]
    toolchains: Vec<String>,
}

pub fn exec(_env: &dyn Environment, opts: ToolchainInstall) -> DfxResult {
    for tc in opts.toolchains {
        let _tcd = tc.parse::<ToolchainDesc>()?;
    }
    Ok(())
}
