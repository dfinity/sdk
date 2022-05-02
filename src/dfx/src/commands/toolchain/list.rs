use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::toolchain;

use anyhow::Context;
use clap::Parser;

/// List installed toolchains
#[derive(Parser)]
#[clap(name("list"))]
pub struct ToolchainList {}

pub fn exec(_env: &dyn Environment, _opts: ToolchainList) -> DfxResult {
    let toolchains =
        toolchain::list_installed_toolchains().context("Failed to get installed toolchains.")?;
    for toolchain in toolchains {
        println!("{}", toolchain);
    }
    Ok(())
}
