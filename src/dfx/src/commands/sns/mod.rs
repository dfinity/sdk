//! Command line interface for `dfx sns`.
#![warn(clippy::missing_docs_in_private_items)]
use crate::{
    commands::sns::config::SnsConfigOpts,
    commands::sns::download::SnsDownloadOpts,
    commands::sns::import::SnsImportOpts,
    lib::{environment::Environment, error::DfxResult},
};

use clap::Parser;

mod config;
mod deploy;
mod download;
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
    Config(SnsConfigOpts),
    /// Subcommand for creating an SNS.
    Deploy(deploy::DeployOpts),
    /// Subcommand for importing sns API definitions and canister IDs.
    Import(SnsImportOpts),
    /// Subcommand for downloading SNS WASMs.
    Download(SnsDownloadOpts),
}

/// Executes `dfx sns` and its subcommands.
pub fn exec(env: &dyn Environment, cmd: SnsOpts) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Config(v) => config::exec(env, v),
        SubCommand::Import(v) => import::exec(env, v),
        SubCommand::Deploy(v) => deploy::exec(env, v),
        SubCommand::Download(v) => download::exec(env, v),
    }
}
