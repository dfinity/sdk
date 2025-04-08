use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use candid::Principal;
use clap::Parser;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};
use slog::info;

/// Sets the identifier of a canister.
#[derive(Parser)]
pub struct CanisterSetIdOpts {
    /// Specifies the name of the canister.
    canister: String,

    /// Specifies the id of the canister.
    id: Principal,

    #[command(flatten)]
    network: NetworkOpt,
}

pub async fn exec(env: &dyn Environment, opts: CanisterSetIdOpts) -> DfxResult {
    env.get_config_or_anyhow()?;
    let log = env.get_logger();
    let network_descriptor = create_network_descriptor(
        env.get_config()?,
        env.get_networks_config(),
        opts.network.to_network_name(),
        None,
        LocalBindDetermination::AsConfigured,
    )?;
    let canister_id_store =
        CanisterIdStore::new(env.get_logger(), &network_descriptor, env.get_config()?)?;

    canister_id_store.add(log, &opts.canister, &opts.id.to_string(), None)?;
    info!(
        log,
        "Set canister id for {} to {}",
        opts.canister,
        opts.id.to_string()
    );
    Ok(())
}
