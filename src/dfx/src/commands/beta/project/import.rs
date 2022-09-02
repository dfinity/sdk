use crate::lib::error::DfxResult;
use crate::lib::project::import::import_canister_definitions;
use crate::lib::project::network_mappings::get_network_mappings;
use crate::Environment;

use clap::Parser;
use tokio::runtime::Runtime;

/// Imports canister definitions from another project, as remote canisters
#[derive(Parser)]
pub struct ImportOpts {
    /// Path to dfx.json (local file path or url)
    location: String,

    /// Specifies the canister name. Either this or the --all flag are required.
    canister_name: Option<String>,

    /// Imports all canisters found in the other project.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,

    /// An optional prefix for canisters names to add to the project
    #[clap(long)]
    prefix: Option<String>,

    /// Networks to import canisters ids for.
    ///   --network-mapping <network name in both places>
    ///   --network-mapping <network name here>=<network name in project being imported>
    /// Examples:
    ///   --network-mapping ic
    ///   --network-mapping ic=mainnet
    #[clap(long, default_value = "ic", multiple_occurrences(true))]
    network_mapping: Vec<String>,
}

pub fn exec(env: &dyn Environment, opts: ImportOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let mut config = config.as_ref().clone();

    let network_mappings = get_network_mappings(&opts.network_mapping)?;

    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        import_canister_definitions(
            env.get_logger(),
            &mut config,
            &opts.location,
            opts.prefix.as_deref(),
            opts.canister_name,
            &network_mappings,
        )
        .await
    })
}
