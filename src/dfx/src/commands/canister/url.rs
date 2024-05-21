use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use crate::util::url::{construct_frontend_url, construct_ui_canister_url};
use candid::Principal;
use clap::Parser;
use console::Style;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};

// Prints the url of a canister.
#[derive(Parser)]
pub struct CanisterUrlOpts {
    /// Specifies the name or id of the canister.
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

    // Get canister id, try to parse first as users can input canister id or canister name.
    let canister_arg = opts.canister.as_str();
    let canister_id =
        Principal::from_text(canister_arg).or_else(|_| canister_id_store.get(canister_arg))?;

    let canister_id_text = Principal::to_text(&canister_id);
    if let Some(canister_name) = canister_id_store.get_name(&canister_id_text) {
        if let Some(canisters) = &config.get_config().canisters {
            if let Some(canister_config) = canisters.get(canister_name) {
                // Ignore if it's remote canister.
                if config
                    .get_config()
                    .is_remote_canister(&canister_name, &network_descriptor.name)?
                {
                    return Ok(());
                }

                let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;
                let green = Style::new().green();

                // Display as frontend if it's an asset canister, or custome type with a frontend.
                if canister_info.is_assets() || canister_config.frontend.is_some() {
                    let (canister_url, url2) =
                        construct_frontend_url(&network_descriptor, &canister_id)?;
                    println!("{}", green.apply_to(canister_url));
                    if let Some(canister_url2) = url2 {
                        println!("{}", green.apply_to(canister_url2));
                    }
                }

                // Display as backend.
                if !canister_info.is_assets() {
                    let url = construct_ui_canister_url(env, &canister_id)?;
                    if let Some(canister_url) = url {
                        println!("{}", green.apply_to(canister_url));
                    }
                }
            }
        }
    }

    Ok(())
}
