use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::toolchain;
use crate::lib::toolchain::Toolchain;

use clap::Clap;

/// Set default toolchain or get current default toolchain
#[derive(Clap)]
#[clap(name("default"))]
pub struct ToolchainDefault {
    /// Toolchain name, such as '0.6.22', '0.6', 'latest'
    toolchain: Option<String>,
}

pub fn exec(_env: &dyn Environment, opts: ToolchainDefault) -> DfxResult {
    match opts.toolchain {
        Some(name) => {
            let toolchain = name.parse::<Toolchain>()?;
            toolchain.set_default()?;
        }
        None => {
            let toolchain = toolchain::get_default_toolchain()?;
            println!("{}", toolchain);
        }
    }
    Ok(())
}
