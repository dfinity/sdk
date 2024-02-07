use crate::lib::dfxvm::dfxvm_released;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use anyhow::bail;
use clap::Parser;
use console::Style;
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
    if dfxvm_released()? {
        println!("You can install dfxvm by running the following command:");
        println!();
        let command = Style::new()
            .cyan()
            .apply_to(r#"sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)""#);
        println!("    {command}");
    } else {
        println!("For installation instructions, see:");
        let url = Style::new()
            .green()
            .apply_to("https://github.com/dfinity/dfxvm/blob/main/README.md");
        println!("    {url}");
    }
    println!();
    bail!("toolchain command removed");
}
