use crate::actors::{
    start_btc_adapter_actor, start_canister_http_adapter_actor, start_emulator_actor,
    start_replica_actor, start_shutdown_controller,
};
use crate::commands::start::{
    apply_command_line_parameters, configure_btc_adapter_if_enabled,
    configure_canister_http_adapter_if_enabled, empty_writable_path,
};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::id::write_network_id;
use crate::lib::replica_config::{HttpHandlerConfig, ReplicaConfig};
use dfx_core::config::model::dfinity::DEFAULT_REPLICA_PORT;
use dfx_core::config::model::local_server_descriptor::LocalServerDescriptor;
use dfx_core::json::{load_json_file, save_json_file};
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};
use anyhow::{bail, Context};
use clap::{ArgAction, Parser};
use fn_error_context::context;
use slog::warn;
use std::default::Default;
use std::fs;
use std::fs::create_dir_all;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use super::start::CachedConfig;

/// Starts a local Internet Computer replica.
#[derive(Parser)]
pub struct ReplicaOpts {
    /// Specifies the port the local replica should listen to.
    #[arg(long)]
    port: Option<String>,

    /// Runs a dedicated emulator instead of the replica
    #[arg(long)]
    emulator: bool,

    /// Address of bitcoind node.  Implies --enable-bitcoin.
    #[arg(long, conflicts_with("emulator"), action = ArgAction::Append)]
    bitcoin_node: Vec<SocketAddr>,

    /// enable bitcoin integration
    #[arg(long, conflicts_with("emulator"))]
    enable_bitcoin: bool,

    /// enable canister http requests
    #[arg(long, conflicts_with("emulator"))]
    enable_canister_http: bool,

    /// The delay (in milliseconds) an update call should take. Lower values may be expedient in CI.
    #[arg(long, conflicts_with("emulator"), default_value_t = 600)]
    artificial_delay: u32,

    /// Start even if the network config was modified.
    #[arg(long)]
    force: bool,
}

/// Gets the configuration options for the Internet Computer replica.
#[context("Failed to get replica config.")]
fn get_config(
    local_server_descriptor: &LocalServerDescriptor,
    replica_port_path: PathBuf,
    state_root: &Path,
    artificial_delay: u32,
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
        artificial_delay,
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
        artificial_delay,
        force,
    }: ReplicaOpts,
) -> DfxResult {
    warn!(
        env.get_logger(),
        "The replica command is deprecated. \
        Please use the start command instead. \
        If you have a good reason to use the replica command, \
        please contribute to the discussion at https://github.com/dfinity/sdk/discussions/3163"
    );
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
    local_server_descriptor.describe(env.get_logger());

    let temp_dir = &local_server_descriptor.data_directory;
    create_dir_all(temp_dir).with_context(|| {
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

    let previous_config_path = local_server_descriptor.effective_config_path();

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
    let replica_config = {
        let mut replica_config = get_config(
            local_server_descriptor,
            replica_port_path,
            &state_root,
            artificial_delay,
        )?;
        if let Some(btc_adapter_config) = btc_adapter_config.as_ref() {
            replica_config = replica_config.with_btc_adapter_enabled();
            if let Some(btc_adapter_socket) = btc_adapter_config.get_socket_path() {
                replica_config = replica_config.with_btc_adapter_socket(btc_adapter_socket);
            }
        }
        if let Some(canister_http_adapter_config) = canister_http_adapter_config.as_ref() {
            replica_config = replica_config.with_canister_http_adapter_enabled();
            if let Some(socket_path) = canister_http_adapter_config.get_socket_path() {
                replica_config = replica_config.with_canister_http_adapter_socket(socket_path);
            }
        }
        replica_config
    };

    let effective_config = if emulator {
        CachedConfig::emulator()
    } else {
        CachedConfig::replica(&replica_config)
    };
    if !force && previous_config_path.exists() {
        let previous_config = load_json_file(&previous_config_path)
            .context("Failed to read replica configuration. Run `dfx start` with `--clean`.")?;
        if effective_config != previous_config {
            bail!("The network configuration was changed. Run `dfx start` with `--clean`.")
        }
    }
    save_json_file(&previous_config_path, &effective_config)
        .context("Failed to write replica configuration")?;

    system.block_on(async move {
        let shutdown_controller = start_shutdown_controller(env)?;
        if emulator {
            start_emulator_actor(
                env,
                local_server_descriptor,
                shutdown_controller,
                emulator_port_path,
            )?;
        } else {
            let btc_adapter_ready_subscribe = btc_adapter_config
                .map(|btc_adapter_config| {
                    start_btc_adapter_actor(
                        env,
                        btc_adapter_config_path,
                        btc_adapter_config.get_socket_path(),
                        shutdown_controller.clone(),
                        btc_adapter_pid_file_path,
                    )
                })
                .transpose()?;
            let canister_http_adapter_ready_subscribe = canister_http_adapter_config
                .map(|canister_http_adapter_config| {
                    start_canister_http_adapter_actor(
                        env,
                        canister_http_adapter_config_path,
                        canister_http_adapter_config.get_socket_path(),
                        shutdown_controller.clone(),
                        canister_http_adapter_pid_file_path,
                    )
                })
                .transpose()?;

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
