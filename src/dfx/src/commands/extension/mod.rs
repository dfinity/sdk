use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::{Parser, Subcommand};

mod install;
mod list;
pub mod run;
mod uninstall;

/// Manages canisters deployed on a network replica.
#[derive(Parser)]
#[clap(name("extension"))]
pub struct ExtensionOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    /// Install an extension
    Install(install::InstallOpts),
    /// Unintall an extension
    Uninstall(uninstall::UninstallOpts),
    /// Executes an extension
    Run(run::RunOpts),
    /// Lists installed extensions
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
