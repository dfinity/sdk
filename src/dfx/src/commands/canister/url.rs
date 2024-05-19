use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use candid::Principal;
use clap::Parser;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};
use super::super::deploy;

// Prints the url of a canister.
#[derive(Parser)]
pub struct CanisterUrlOpts {
    /// Specifies the name of the canister.
    canister: String,

    #[command(flatten)]
    network: NetworkOpt,
}

pub async fn exec(env: &dyn Environment, opts: CanisterUrlOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;

    let network_descriptor = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        opts.network.to_network_name(),
        None,
        LocalBindDetermination::AsConfigured,
    )?;

    let canister_id_store = 
        CanisterIdStore::new(env.get_logger(), &network_descriptor, env.get_config()?)?;

    let canister_name = opts.canister.as_str();
    let canister_id = 
        Principal::from_text(canister_name).or_else(|_| canister_id_store.get(canister_name))?;

    let canister_id_text = Principal::to_text(&canister_id);

    if let Some(name) = canister_id_store.get_name(&canister_id_text) {
        if let Some(canisters) = &config.get_config().canisters {
            if let Some(canister_config) = canisters.get(name) {
                // TODO : Handle remote.

                let canister_info = CanisterInfo::load(&config, name, Some(canister_id))?;

                let is_assets = canister_info.is_assets() || canister_config.frontend.is_some();

                if is_assets {
                    let (canister_url, url2) = deploy::construct_frontend_url(&network_descriptor, &canister_id)?;
                    println!("{}", canister_url);
                    if let Some(canister_url2) = url2 {
                        println!("{}", canister_url2);
                    }
                }

                if !canister_info.is_assets() {
                    let url = deploy::construct_ui_canister_url(env, &canister_id)?;
                    if let Some(canister_url) = url {
                        println!("{}", canister_url);
                    }
                }
            }
        }
    }

    Ok(())
}
