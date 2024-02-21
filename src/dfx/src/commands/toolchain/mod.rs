use crate::lib::dfxvm::display_dfxvm_installation_instructions;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use anyhow::bail;
use clap::Parser;
use std::ffi::OsString;

/// Manage the dfx toolchains (obsolete)
#[derive(Parser)]
#[command(name = "toolchain", disable_help_flag = true)]
pub struct ToolchainOpts {
    #[arg(allow_hyphen_values = true)]
    _params: Vec<OsString>,
}

pub fn exec(_env: &dyn Environment, _opts: ToolchainOpts) -> DfxResult {
    println!("The toolchain command has been removed.");
    println!("Please use the dfx version manager (dfxvm) to manage dfx versions.");
    println!();
    display_dfxvm_installation_instructions();
    println!();
    bail!("toolchain command removed");
}
