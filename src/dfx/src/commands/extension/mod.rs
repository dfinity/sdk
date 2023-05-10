#![allow(dead_code)]

use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::{Parser, Subcommand};

mod install;
mod list;
pub mod run;
mod uninstall;

/// Manages the dfx extensions.
#[derive(Parser)]
#[command(name = "extension")]
pub struct ExtensionOpts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    /// Install an extension.
    Install(install::InstallOpts),
    /// Uninstall an extension.
    Uninstall(uninstall::UninstallOpts),
    /// Execute an extension.
    Run(run::RunOpts),
    /// List installed extensions.
    List,
}

pub fn exec(env: &dyn Environment, opts: ExtensionOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::Install(v) => install::exec(env, v),
        SubCommand::Uninstall(v) => uninstall::exec(env, v),
        SubCommand::Run(v) => run::exec(env, v),
        SubCommand::List => list::exec(env),
    }
}
