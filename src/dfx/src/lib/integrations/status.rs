use crate::lib::error::DfxResult;
use crate::lib::integrations::bitcoin::MAINNET_BITCOIN_CANISTER_ID;
use crate::lib::integrations::{create_integrations_agent, wait_for_canister_installed};
use dfx_core::config::model::local_server_descriptor::LocalServerDescriptor;

use slog::Logger;

pub async fn wait_for_integrations_initialized(
    agent_url: &str,
    logger: &Logger,
    local_server_descriptor: &LocalServerDescriptor,
) -> DfxResult {
    if !local_server_descriptor.bitcoin.enabled {
        return Ok(());
    }

    let agent = create_integrations_agent(agent_url, logger).await?;

    if local_server_descriptor.bitcoin.enabled {
        wait_for_canister_installed(&agent, &MAINNET_BITCOIN_CANISTER_ID).await?;
    }

    Ok(())
}
