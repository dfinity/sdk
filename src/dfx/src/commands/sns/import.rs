//! Code for the command line `dfx sns import`
use crate::lib::error::DfxResult;
use crate::lib::info::replica_rev;
use crate::lib::project::import::import_canister_definitions;
use crate::lib::project::network_mappings::get_network_mappings;
use crate::Environment;

use clap::Parser;
use tokio::runtime::Runtime;

/// Imports the sns canisters
#[derive(Parser)]
pub struct SnsImportOpts {
    /// Networks to import canisters ids for.
    ///   --network-mapping <network name in both places>
    ///   --network-mapping <network name here>=<network name in project being imported>
    /// Examples:
    ///   --network-mapping ic
    ///   --network-mapping ic=mainnet
    #[clap(long, default_value = "ic=mainnet", multiple_occurrences(true))]
    network_mapping: Vec<String>,
}

/// Executes the command line `dfx sns import`.
pub fn exec(env: &dyn Environment, opts: SnsImportOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let mut config = config.as_ref().clone();

    let network_mappings = get_network_mappings(&opts.network_mapping)?;

    let runtime = Runtime::new().expect("Unable to create a runtime");
    let ic_commit = std::env::var("DFX_IC_COMMIT").unwrap_or_else(|_| replica_rev().to_string());
    let their_dfx_json_location =
        format!("https://raw.githubusercontent.com/dfinity/ic/{ic_commit}/rs/sns/cli/dfx.json");
    runtime.block_on(import_canister_definitions(
        env.get_logger(),
        &mut config,
        &their_dfx_json_location,
        None,
        None,
        &network_mappings,
    ))
}
