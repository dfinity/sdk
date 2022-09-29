//! Command line interface for `dfx sns`.
#![warn(clippy::missing_docs_in_private_items)]
use crate::{
    commands::sns::config::SnsConfigOpts,
    commands::sns::import::SnsImportOpts,
    lib::{environment::Environment, error::DfxResult},
};

use clap::Parser;

mod config;
mod deploy;
mod import;

/// Options for `dfx sns`.
#[derive(Parser)]
#[command(name = "sns")]
pub struct SnsOpts {
    /// Arguments and flags for subcommands.
    #[command(subcommand)]
    subcmd: SubCommand,
}

/// Subcommands of `dfx sns`
#[derive(Parser)]
enum SubCommand {
    /// Subcommands for working with configuration.
    #[command(hide(true))]
    Config(SnsConfigOpts),
    /// Subcommand for creating an SNS.
    #[command(hide(true))]
    Deploy(deploy::DeployOpts),
    /// Subcommand for importing sns API definitions and canister IDs.
    #[command(hide(true))]
    Import(SnsImportOpts),
}

/// Executes `dfx sns` and its subcommands.
pub fn exec(env: &dyn Environment, cmd: SnsOpts) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Config(v) => config::exec(env, v),
        SubCommand::Import(v) => import::exec(env, v),
        SubCommand::Deploy(v) => deploy::exec(env, v),
    }
}
