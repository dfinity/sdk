use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use candid::Principal;
use clap::Parser;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};

/// Prints the identifier of a canister.
#[derive(Parser)]
pub struct CanisterIdOpts {
    /// Specifies the name of the canister.
    canister: String,

    #[command(flatten)]
    network: NetworkOpt,
}

pub async fn exec(env: &dyn Environment, opts: CanisterIdOpts) -> DfxResult {
    env.get_config_or_anyhow()?;
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
    println!("{}", Principal::to_text(&canister_id));
    Ok(())
}
