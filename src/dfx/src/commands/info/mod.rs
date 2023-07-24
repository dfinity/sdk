mod replica_port;
mod webserver_port;
use crate::commands::info::replica_port::get_replica_port;
use crate::commands::info::webserver_port::get_webserver_port;
use crate::lib::error::DfxResult;
use crate::lib::info;
use crate::Environment;
use anyhow::Context;
use clap::{Parser, Subcommand};
use dfx_core::config::model::dfinity::NetworksConfig;

#[derive(Subcommand, Clone, Debug)]
enum InfoType {
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
}

pub fn exec(env: &dyn Environment, opts: InfoOpts) -> DfxResult {
    let value = match opts.info_type {
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
