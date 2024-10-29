use crate::{lib::error::DfxResult, Environment};
use anyhow::anyhow;
use dfx_core::network::root_key;

pub async fn fetch_root_key_if_needed(env: &dyn Environment) -> DfxResult {
    let agent = env.get_agent();
    let network = env.get_network_descriptor();
    root_key::fetch_root_key_when_non_mainnet(agent, network).await
        .map_err(|e| anyhow!("Failed to fetch the root key, did you run 'dfx start' to start the local replica?\n{}", e))?;
    Ok(())
}

/// Fetches the root key of the local network.
/// Returns an error if attempted to run on the real IC.
pub async fn fetch_root_key_or_anyhow(env: &dyn Environment) -> DfxResult {
    let agent = env.get_agent();
    let network = env.get_network_descriptor();
    root_key::fetch_root_key_when_non_mainnet_or_error(agent, network).await?;
    Ok(())
}
