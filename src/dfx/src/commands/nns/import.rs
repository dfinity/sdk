use crate::{DfxResult, Environment};

use crate::lib::project::import::import_canister_definitions;
use crate::lib::project::network_mappings::get_network_mappings;
use clap::Parser;

/// Imports the nns canisters
#[derive(Parser)]
pub struct ImportOpts {
    /// Networks to import canisters ids for.
    ///   --network-mapping <network name in both places>
    ///   --network-mapping <network name here>=<network name in project being imported>
    /// Examples:
    ///   --network-mapping ic
    ///   --network-mapping ic=mainnet
    #[clap(long, default_value = "ic=mainnet", multiple_occurrences(true))]
    network_mapping: Vec<String>,
}

pub async fn exec(env: &dyn Environment, opts: ImportOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let mut config = config.as_ref().clone();

    let network_mappings = get_network_mappings(&opts.network_mapping)?;

    import_canister_definitions(
        env.get_logger(),
        &mut config,
        "https://raw.githubusercontent.com/dfinity/ic/master/rs/nns/dfx.json",
        Some("nns-"),
        None,
        &network_mappings,
    )
    .await
}
