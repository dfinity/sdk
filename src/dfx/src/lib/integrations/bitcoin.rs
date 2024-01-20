use crate::actors::replica::BitcoinIntegrationConfig;
use crate::lib::error::DfxResult;
use crate::lib::integrations::initialize_integration_canister;
use crate::util::assets::bitcoin_wasm;
use candid::Principal;
use fn_error_context::context;
use ic_agent::Agent;
use slog::{debug, Logger};

pub const MAINNET_BITCOIN_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x01, 0xA0, 0x00, 0x01, 0x01, 0x01]);

#[context("Failed to initialize bitcoin canister")]
pub async fn initialize_bitcoin_canister(
    agent: &Agent,
    logger: &Logger,
    bitcoin_integration_config: BitcoinIntegrationConfig,
) -> DfxResult {
    debug!(logger, "Initializing bitcoin canister");

    let name = "bitcoin integration";
    let canister_id = MAINNET_BITCOIN_CANISTER_ID;
    let wasm = bitcoin_wasm(logger)?;
    let init_arg = &bitcoin_integration_config.canister_init_arg;

    initialize_integration_canister(agent, logger, name, canister_id, &wasm, init_arg).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_canister_id() {
        assert_eq!(
            MAINNET_BITCOIN_CANISTER_ID,
            Principal::from_text("g4xu7-jiaaa-aaaan-aaaaq-cai").unwrap()
        );
    }
}
