use crate::actors::{
    start_btc_adapter_actor, start_canister_http_adapter_actor, start_emulator_actor,
    start_replica_actor, start_shutdown_controller,
};
use crate::commands::start::{
    apply_command_line_parameters, configure_btc_adapter_if_enabled,
    configure_canister_http_adapter_if_enabled, empty_writable_path,
};
use crate::config::dfinity::DEFAULT_REPLICA_PORT;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::id::write_network_id;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use crate::lib::provider::{create_network_descriptor, LocalBindDetermination};
use crate::lib::replica_config::{HttpHandlerConfig, ReplicaConfig};

use anyhow::Context;
use clap::Parser;
use fn_error_context::context;
use std::default::Default;
use std::fs;
use std::fs::create_dir_all;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

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

    /// enable bitcoin integration
    #[clap(long, conflicts_with("emulator"))]
    enable_bitcoin: bool,

    /// enable canister http requests
    #[clap(long, conflicts_with("emulator"))]
    enable_canister_http: bool,
}

/// Gets the configuration options for the Internet Computer replica.
#[context("Failed to get replica config.")]
fn get_config(
    local_server_descriptor: &LocalServerDescriptor,
    replica_port_path: PathBuf,
    state_root: &Path,
) -> DfxResult<ReplicaConfig> {
    let config = &local_server_descriptor.replica;
    let port = config.port.unwrap_or(DEFAULT_REPLICA_PORT);
    let mut http_handler: HttpHandlerConfig = Default::default();
    if port == 0 {
        http_handler.write_port_to = Some(replica_port_path);
    } else {
        http_handler.port = Some(port);
    };

    let mut replica_config = ReplicaConfig::new(
        state_root,
        config.subnet_type.unwrap_or_default(),
        config.log_level.unwrap_or_default(),
    );
    replica_config.http_handler = http_handler;

    Ok(replica_config)
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment), the proxy, and (if configured) the bitcoin adapter.
pub fn exec(
    env: &dyn Environment,
    ReplicaOpts {
        port,
        emulator,
        bitcoin_node,
        enable_bitcoin,
        enable_canister_http,
    }: ReplicaOpts,
) -> DfxResult {
    let system = actix::System::new();

    let network_descriptor = create_network_descriptor(
        env.get_config(),
        env.get_networks_config(),
        None,
        Some(env.get_logger().clone()),
        LocalBindDetermination::AsConfigured,
    )?;
    let network_descriptor = apply_command_line_parameters(
        env.get_logger(),
        network_descriptor,
        None,
        port,
        enable_bitcoin,
        bitcoin_node,
        enable_canister_http,
    )?;

    let local_server_descriptor = network_descriptor.local_server_descriptor()?;
    local_server_descriptor.describe(env.get_logger(), true, true);

    let temp_dir = &local_server_descriptor.data_directory;
    create_dir_all(&temp_dir).with_context(|| {
        format!(
            "Failed to create network temp directory {}.",
            temp_dir.to_string_lossy()
        )
    })?;
    if !local_server_descriptor.network_id_path().exists() {
        write_network_id(local_server_descriptor)?;
    }

    let btc_adapter_pid_file_path =
        empty_writable_path(local_server_descriptor.btc_adapter_pid_path())?;
    let btc_adapter_config_path =
        empty_writable_path(local_server_descriptor.btc_adapter_config_path())?;
    let btc_adapter_socket_holder_path = local_server_descriptor.btc_adapter_socket_holder_path();
    let canister_http_adapter_pid_file_path =
        empty_writable_path(local_server_descriptor.canister_http_adapter_pid_path())?;
    let canister_http_adapter_config_path =
        empty_writable_path(local_server_descriptor.canister_http_adapter_config_path())?;
    let canister_http_adapter_socket_holder_path =
        local_server_descriptor.canister_http_adapter_socket_holder_path();

    // dfx bootstrap will read these port files to find out which port to use,
    // so we need to make sure only one has a valid port in it.
    let replica_config_dir = local_server_descriptor.replica_configuration_dir();
    fs::create_dir_all(&replica_config_dir).with_context(|| {
        format!(
            "Failed to create replica config directory {}.",
            replica_config_dir.display()
        )
    })?;
    let replica_port_path = empty_writable_path(local_server_descriptor.replica_port_path())?;
    let emulator_port_path = empty_writable_path(local_server_descriptor.ic_ref_port_path())?;
    let state_root = local_server_descriptor.state_dir();

    let btc_adapter_config = configure_btc_adapter_if_enabled(
        local_server_descriptor,
        &btc_adapter_config_path,
        &btc_adapter_socket_holder_path,
    )?;
    let btc_adapter_socket_path = btc_adapter_config
        .as_ref()
        .and_then(|cfg| cfg.get_socket_path());

    let canister_http_adapter_config = configure_canister_http_adapter_if_enabled(
        local_server_descriptor,
        &canister_http_adapter_config_path,
        &canister_http_adapter_socket_holder_path,
    )?;
    let canister_http_socket_path = canister_http_adapter_config
        .as_ref()
        .and_then(|cfg| cfg.get_socket_path());
    let mut replica_config = get_config(local_server_descriptor, replica_port_path, &state_root)?;

    system.block_on(async move {
        let shutdown_controller = start_shutdown_controller(env)?;
        if emulator {
            start_emulator_actor(env, shutdown_controller, emulator_port_path)?;
        } else {
            let (btc_adapter_ready_subscribe, btc_adapter_socket_path) =
                if let Some(ref btc_adapter_config) = btc_adapter_config {
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
            let (canister_http_adapter_ready_subscribe, canister_http_socket_path) =
                if let Some(ref canister_http_adapter_config) = canister_http_adapter_config {
                    let socket_path = canister_http_adapter_config.get_socket_path();
                    let ready_subscribe = start_canister_http_adapter_actor(
                        env,
                        canister_http_adapter_config_path,
                        socket_path.clone(),
                        shutdown_controller.clone(),
                        canister_http_adapter_pid_file_path,
                    )?
                    .recipient();
                    (Some(ready_subscribe), socket_path)
                } else {
                    (None, None)
                };

            if btc_adapter_config.is_some() {
                replica_config = replica_config.with_btc_adapter_enabled();
                if let Some(btc_adapter_socket) = btc_adapter_socket_path {
                    replica_config = replica_config.with_btc_adapter_socket(btc_adapter_socket);
                }
            }
            if canister_http_adapter_config.is_some() {
                replica_config = replica_config.with_canister_http_adapter_enabled();
                if let Some(socket_path) = canister_http_socket_path {
                    replica_config = replica_config.with_canister_http_adapter_socket(socket_path);
                }
            }

            start_replica_actor(
                env,
                replica_config,
                local_server_descriptor,
                shutdown_controller,
                btc_adapter_ready_subscribe,
                canister_http_adapter_ready_subscribe,
            )?;
        }
        DfxResult::Ok(())
    })?;
    system.run()?;

    if let Some(btc_adapter_socket_path) = btc_adapter_socket_path {
        let _ = std::fs::remove_file(&btc_adapter_socket_path);
    }
    if let Some(canister_http_socket_path) = canister_http_socket_path {
        let _ = std::fs::remove_file(&canister_http_socket_path);
    }

    Ok(())
}
