mod replica_port;
mod webserver_port;
use crate::commands::info::{replica_port::get_replica_port, webserver_port::get_webserver_port};
use crate::lib::agent::create_anonymous_agent_environment;
use crate::lib::error::DfxResult;
use crate::lib::info;
use crate::lib::named_canister::get_ui_canister_url;
use crate::lib::network::network_opt::NetworkOpt;
use crate::Environment;
use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use dfx_core::config::model::dfinity::NetworksConfig;

#[derive(Subcommand, Clone, Debug)]
enum InfoType {
    /// Show the URL of the Candid UI canister
    CandidUiUrl,
    /// Show the port of the local replica
    ReplicaPort,
    /// Show the revision of the replica shipped with this dfx binary
    ReplicaRev,
    /// Show the port of the webserver
    WebserverPort,
    /// Show the path to network configuration file
    NetworksJsonPath,
}

#[derive(Parser)]
#[command(name = "info")]
/// Get information about the replica shipped with dfx, path to networks.json, and network ports of running replica.
pub struct InfoOpts {
    #[command(subcommand)]
    info_type: InfoType,

    #[command(flatten)]
    network: NetworkOpt,
}

pub fn exec(env: &dyn Environment, opts: InfoOpts) -> DfxResult {
    let value = match opts.info_type {
        InfoType::CandidUiUrl => {
            let env = create_anonymous_agent_environment(env, opts.network.to_network_name())?;
            match get_ui_canister_url(&env)? {
                Some(url) => url.to_string(),
                None => bail!(
                    "Candid UI not installed on network {}.",
                    env.get_network_descriptor().name
                ),
            }
        }
        InfoType::ReplicaPort => get_replica_port(env)?,
        InfoType::ReplicaRev => info::replica_rev().to_string(),
        InfoType::WebserverPort => get_webserver_port(env)?,
        InfoType::NetworksJsonPath => NetworksConfig::new()?
            .get_path()
            .to_str()
            .context("Failed to convert networks.json path to a string.")?
            .to_string(),
    };
    println!("{}", value);
    Ok(())
}
