//! Code for the command line: `dfx nns install`
use crate::lib::error::DfxResult;
use crate::Environment;
use anyhow::anyhow;

use crate::lib::nns::install_nns::{get_and_check_replica_url, get_with_retries, install_nns};
use crate::lib::root_key::fetch_root_key_if_needed;
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
#[clap(about)]
pub struct InstallOpts {}

/// Executes `dfx nns install`.
pub async fn exec(env: &dyn Environment, _opts: InstallOpts) -> DfxResult {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    // Wait for the server to be ready...
    let nns_url = get_and_check_replica_url(env)?;
    get_with_retries(&nns_url).await?;

    fetch_root_key_if_needed(env).await?;

    let ic_nns_init_path = env.get_cache().get_binary_command_path("ic-nns-init")?;

    install_nns(env, agent, &ic_nns_init_path).await
}
