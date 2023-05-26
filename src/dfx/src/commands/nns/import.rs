//! Code for the command line: `dfx nns import`
use std::collections::BTreeMap;

use crate::lib::error::DfxResult;
use crate::lib::info::replica_rev;
use crate::lib::nns::install_nns::canisters::{NNS_CORE, NNS_FRONTEND};
use crate::lib::project::import::{
    get_canisters_json_object, import_canister_definitions, set_remote_canister_ids,
    ImportNetworkMapping,
};
use crate::lib::project::network_mappings::get_network_mappings;
use crate::Environment;
use dfx_core::config::model::canister_id_store::CanisterIds;
use dfx_core::config::model::dfinity::Config;

use clap::{ArgAction, Parser};
use slog::{info, Logger};

/// Imports the nns canisters
#[derive(Parser)]
pub struct ImportOpts {
    /// Networks to import canisters ids for.
    ///   --network-mapping <network name in both places>
    ///   --network-mapping <network name here>=<network name in project being imported>
    /// Examples:
    ///   --network-mapping ic
    ///   --network-mapping ic=mainnet
    #[arg(long, default_value = "ic=mainnet", action = ArgAction::Append)]
    network_mapping: Vec<String>,
}

/// Executes `dfx nns import`
pub async fn exec(env: &dyn Environment, opts: ImportOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let mut config = config.as_ref().clone();
    let logger = env.get_logger();

    let network_mappings = get_network_mappings(&opts.network_mapping)?;

    // Import core NNS canisters
    let ic_commit = std::env::var("DFX_IC_COMMIT").unwrap_or_else(|_| replica_rev().to_string());
    let dfx_url_str = {
        let ic_project = std::env::var("DFX_IC_SRC").unwrap_or_else(|_| {
            format!("https://raw.githubusercontent.com/dfinity/ic/{ic_commit}")
        });
        format!("{ic_project}/rs/nns/dfx.json")
    };
    import_canister_definitions(
        logger,
        &mut config,
        &dfx_url_str,
        Some("nns-"),
        None,
        &network_mappings,
    )
    .await?;

    // Import frontend NNS canisters
    // TODO: The version of nns-dapp deployed by dfx nns install is very old and needs to be
    // updated and parameterized.
    //       - The pattern of where assets are has changed
    //       - The wasm can now be used on any network
    //       - Deploying the new nns-dapp requires passing arguments
    // The following URL has the correct canister IDs and uses __default, so should give
    // useful canister IDs in more cases but the wasm is from a much older commit.
    let frontend_url_str = "https://raw.githubusercontent.com/dfinity/nns-dapp/5a9b84ac38ab60065dd40c5174384c4c161875d3/dfx.json"; // TODO: parameterize URL
    for canister_name in ["nns-dapp", "internet_identity"] {
        import_canister_definitions(
            logger,
            &mut config,
            frontend_url_str,
            None,
            Some(canister_name.to_string()),
            &network_mappings,
        )
        .await?;
    }

    set_local_nns_canister_ids(logger, &mut config)
}

/// Sets local canister IDs
/// The "local" entries at the remote URL are often misssing or do not match our NNS installation.
/// Always set the local values per our local NNS deployment.  We have all the information locally.
fn set_local_nns_canister_ids(logger: &Logger, config: &mut Config) -> DfxResult {
    let nns_init_canister_ids = NNS_CORE.iter().map(|canister| {
        (
            canister.canister_name.to_string(),
            BTreeMap::from([("local".to_string(), canister.canister_id.to_string())]),
        )
    });
    let nns_frontend_canister_ids = NNS_FRONTEND.iter().map(|canister| {
        (
            canister.canister_name.to_string(),
            BTreeMap::from([("local".to_string(), canister.canister_id.to_string())]),
        )
    });
    let local_canister_ids: CanisterIds = nns_init_canister_ids
        .chain(nns_frontend_canister_ids)
        .collect();
    let local_mappings = [ImportNetworkMapping {
        network_name_in_this_project: "local".to_string(),
        network_name_in_project_being_imported: "local".to_string(),
    }];

    let canisters = get_canisters_json_object(config)?;

    for canister_name in local_canister_ids.keys() {
        // Not all NNS canisters may be listed in the remote dfx.json
        let dfx_canister = canisters
            .get_mut(canister_name)
            .and_then(|canister_entry| canister_entry.as_object_mut());
        // If the canister is in dfx.json, set the local canister ID.
        if let Some(dfx_canister) = dfx_canister {
            set_remote_canister_ids(
                logger,
                canister_name,
                &local_mappings,
                &local_canister_ids,
                dfx_canister,
            )?;
        } else {
            info!(logger, "{} has no local canister ID.", canister_name);
        }
    }
    config.save()?;
    Ok(())
}
