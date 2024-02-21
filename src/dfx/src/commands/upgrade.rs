use crate::lib::dfxvm::display_dfxvm_installation_instructions;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use anyhow::bail;
use clap::Parser;

/// Upgrade DFX (removed: use https://github.com/dfinity/dfxvm instead)
#[derive(Parser)]
pub struct UpgradeOpts {
    /// Current Version.
    #[arg(long)]
    current_version: Option<String>,

    #[arg(long, default_value = "https://sdk.dfinity.org", hide = true)]
    release_root: String,
}

pub fn exec(_env: &dyn Environment, _opts: UpgradeOpts) -> DfxResult {
    println!(
        "dfx upgrade has been removed. Please use the dfx version manager (dfxvm) to upgrade."
    );
    println!();
    display_dfxvm_installation_instructions();
    println!();
    bail!("dfx upgrade has been removed");
}
