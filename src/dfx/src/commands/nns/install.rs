//! Code for the command line: `dfx nns install`
use crate::lib::error::DfxResult;
use crate::lib::nns::install_nns::{get_and_check_replica_url, get_with_retries, install_nns};
use crate::Environment;
use dfx_core::network::root_key::fetch_root_key_when_local;

use anyhow::anyhow;
use clap::Parser;

/// Installs the NNS canisters, Internet Identity and the NNS frontend dapp
///
/// - The core network nervous system canisters are nns-registry, nns-governance, nns-ledger, nns-root, nns-cycles-minting,
///   nns-lifeline, nns-genesis-token and nns-sns-wasm.
///   Source code is at <https://github.com/dfinity/ic/tree/master/rs/nns#network-nervous-system-nns>.
///
///
/// - internet_identity is a login service.
///   Source code is at <https://github.com/dfinity/internet-identity>.
///   This frontend is typically served at: <http://qaa6y-5yaaa-aaaaa-aaafa-cai.localhost:8080>.
///
/// - nns-dapp is a voting app and wallet. Source code is at <https://github.com/dfinity/nns-dapp>.
///   This frontend is typically served at: <http://qhbym-qaaaa-aaaaa-aaafq-cai.localhost:8080>.
#[derive(Parser)]
#[command(about)]
pub struct InstallOpts {
    /// Initialize ledger canister with these test accounts
    #[arg(long, num_args = ..)]
    ledger_accounts: Vec<String>,
}

/// Executes `dfx nns install`.
pub async fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let network_descriptor = env.get_network_descriptor();
    let networks_config = env.get_networks_config();
    let logger = env.get_logger();
    let cache = env.get_cache();

    // Wait for the server to be ready...
    let nns_url = get_and_check_replica_url(network_descriptor, logger)?;
    get_with_retries(&nns_url).await?;

    fetch_root_key_when_local(agent, network_descriptor).await?;

    let ic_nns_init_path = cache.get_binary_command_path("ic-nns-init")?;

    install_nns(
        agent,
        network_descriptor,
        networks_config.as_ref(),
        cache.as_ref(),
        &ic_nns_init_path,
        &opts.ledger_accounts,
        logger,
    )
    .await
}
