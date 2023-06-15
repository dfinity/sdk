//! Code for the command line `dfx sns import`
use crate::lib::error::DfxResult;
use crate::lib::info::replica_rev;
use crate::lib::nns::install_nns::download_sns_wasms;
use crate::Environment;
use std::path::PathBuf;

use clap::Parser;
use tokio::runtime::Runtime;

/// Downloads the SNS canister WASMs
#[derive(Parser)]
pub struct SnsDownloadOpts {
    /// IC commit of SNS canister WASMs to download
    #[arg(long, env = "DFX_IC_COMMIT")]
    ic_commit: Option<String>,
    /// Path to store downloaded SNS canister WASMs
    #[arg(long, default_value = ".")]
    wasms_dir: PathBuf,
}

/// Executes the command line `dfx sns import`.
pub fn exec(_env: &dyn Environment, opts: SnsDownloadOpts) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    let ic_commit = opts.ic_commit.unwrap_or_else(|| replica_rev().to_string());
    runtime.block_on(download_sns_wasms(&ic_commit, &opts.wasms_dir))
}
