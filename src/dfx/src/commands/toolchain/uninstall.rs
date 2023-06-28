use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::toolchain::Toolchain;

use anyhow::Context;
use clap::Parser;

/// Uninstall toolchain(s)
#[derive(Parser)]
#[command(name = "uninstall")]
pub struct ToolchainUninstall {
    /// Toolchain name, such as '0.6.22', '0.6', 'latest'
    #[arg(required = true, num_args = 1..)]
    toolchains: Vec<String>,
}

pub fn exec(_env: &dyn Environment, opts: ToolchainUninstall) -> DfxResult {
    for s in opts.toolchains {
        let toolchain = s
            .parse::<Toolchain>()
            .with_context(|| format!("Failed to parse toolchain name {}.", s))?;
        toolchain.uninstall()?;
    }
    Ok(())
}
