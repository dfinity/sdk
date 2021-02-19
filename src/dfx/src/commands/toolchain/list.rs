use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::toolchain;

use clap::Clap;

/// List installed toolchains
#[derive(Clap)]
#[clap(name("list"))]
pub struct ToolchainList {}

pub fn exec(_env: &dyn Environment, _opts: ToolchainList) -> DfxResult {
    let toolchains = toolchain::list_installed_toolchains()?;
    for toolchain in toolchains {
        println!("{}", toolchain);
    }
    Ok(())
}
