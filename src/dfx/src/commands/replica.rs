use crate::actors::{
    start_btc_adapter_actor, start_emulator_actor, start_replica_actor, start_shutdown_controller,
};
use crate::config::dfinity::ConfigDefaultsReplica;
use crate::error_invalid_argument;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::replica_config::{HttpHandlerConfig, ReplicaConfig};

use crate::commands::start::{configure_btc_adapter_if_enabled, empty_writable_path};
use clap::Parser;
use fn_error_context::context;
use std::default::Default;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Starts a local Internet Computer replica.
#[derive(Parser)]
pub struct ReplicaOpts {
    /// Specifies the port the local replica should listen to.
    #[clap(long)]
    port: Option<String>,

    /// Runs a dedicated emulator instead of the replica
    #[clap(long)]
    emulator: bool,

    /// Address of bitcoind node.  Implies --enable-bitcoin.
    #[clap(long, conflicts_with("emulator"), multiple_occurrences(true))]
    bitcoin_node: Vec<SocketAddr>,

    /// enable the bitcoin adapter
    #[clap(long)]
    enable_bitcoin: bool,
}

/// Gets the configuration options for the Internet Computer replica.
#[context("Failed to get replica config.")]
fn get_config(
    env: &dyn Environment,
    opts: ReplicaOpts,
    btc_adapter_socket: Option<PathBuf>,
) -> DfxResult<ReplicaConfig> {
    let config = get_config_from_file(env);
    let port = get_port(&config, opts.port)?;
    let mut http_handler: HttpHandlerConfig = Default::default();
    if port == 0 {
        let replica_port_path = env
            .get_temp_dir()
            .join("replica-configuration")
            .join("replica-1.port");
        http_handler.write_port_to = Some(replica_port_path);
    } else {
        http_handler.port = Some(port);
    };

    let mut replica_config =
        ReplicaConfig::new(&env.get_state_dir(), config.subnet_type.unwrap_or_default());
    replica_config.http_handler = http_handler;

    if let Some(btc_adapter_socket) = btc_adapter_socket {
        replica_config = replica_config.with_btc_adapter_socket(btc_adapter_socket);
    }
    Ok(replica_config)
}

/// Gets the configuration options for the Internet Computer replica as they were specified in the
/// dfx configuration file.
fn get_config_from_file(env: &dyn Environment) -> ConfigDefaultsReplica {
    env.get_config().map_or(Default::default(), |config| {
        config.get_config().get_defaults().get_replica().to_owned()
    })
}

/// Gets the port number that the Internet Computer replica listens on. First checks if the port
/// number was specified on the command-line using --port, otherwise checks if the port number was
/// specified in the dfx configuration file, otherise defaults to 8080.
#[context("Failed to get port.")]
fn get_port(config: &ConfigDefaultsReplica, port: Option<String>) -> DfxResult<u16> {
    port.map(|port| port.parse())
        .unwrap_or_else(|| {
            let default = 8080;
            Ok(config.port.unwrap_or(default))
        })
        .map_err(|err| error_invalid_argument!("Invalid port number: {}", err))
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment), the proxy, and (if configured) the bitcoin adapter.
pub fn exec(env: &dyn Environment, opts: ReplicaOpts) -> DfxResult {
    let system = actix::System::new();

    let temp_dir = env.get_temp_dir();
    let btc_adapter_pid_file_path = empty_writable_path(temp_dir.join("ic-btc-adapter-pid"))?;
    let btc_adapter_config_path = empty_writable_path(temp_dir.join("ic-btc-adapter-config.json"))?;

    let config = env.get_config_or_anyhow()?;
    let btc_adapter_config = configure_btc_adapter_if_enabled(
        config.get_config(),
        &btc_adapter_config_path,
        opts.enable_bitcoin,
        opts.bitcoin_node.clone(),
    )?;
    let btc_adapter_socket_path = btc_adapter_config
        .as_ref()
        .and_then(|cfg| cfg.get_socket_path());

    system.block_on(async move {
        let shutdown_controller = start_shutdown_controller(env)?;
        if opts.emulator {
            start_emulator_actor(env, shutdown_controller)?;
        } else {
            let (btc_adapter_ready_subscribe, btc_adapter_socket_path) =
                if let Some(btc_adapter_config) = btc_adapter_config {
                    let socket_path = btc_adapter_config.get_socket_path();
                    let ready_subscribe = start_btc_adapter_actor(
                        env,
                        btc_adapter_config_path,
                        socket_path.clone(),
                        shutdown_controller.clone(),
                        btc_adapter_pid_file_path,
                    )?
                    .recipient();
                    (Some(ready_subscribe), socket_path)
                } else {
                    (None, None)
                };

            let replica_config = get_config(env, opts, btc_adapter_socket_path)?;

            start_replica_actor(
                env,
                replica_config,
                shutdown_controller,
                btc_adapter_ready_subscribe,
            )?;
        }
        DfxResult::Ok(())
    })?;
    system.run()?;

    if let Some(btc_adapter_socket_path) = btc_adapter_socket_path {
        let _ = std::fs::remove_file(&btc_adapter_socket_path);
    }

    Ok(())
}
