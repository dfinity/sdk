use crate::lib::error::DfxResult;
use crate::lib::info::replica_rev;
use crate::lib::project::import::import_canister_definitions;
use crate::lib::project::network_mappings::get_network_mappings;
use crate::Environment;

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
    let ic_commit = replica_rev();

    let dfx_url_str = {
        let ic_project = std::env::var("DFX_IC_SRC").unwrap_or_else(|_| {
            format!("https://raw.githubusercontent.com/dfinity/ic/{ic_commit}")
        });
        format!("{ic_project}/rs/nns/dfx.json")
    };
    import_canister_definitions(
        env.get_logger(),
        &mut config,
        &dfx_url_str,
        Some("nns-"),
        None,
        &network_mappings,
    )
    .await
}
