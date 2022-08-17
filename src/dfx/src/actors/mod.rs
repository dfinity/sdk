use crate::actors;
use crate::actors::emulator::Emulator;
use crate::actors::replica::Replica;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::replica_config::ReplicaConfig;

use crate::actors::btc_adapter::signals::BtcAdapterReadySubscribe;
use crate::actors::btc_adapter::BtcAdapter;
use crate::actors::canister_http_adapter::signals::CanisterHttpAdapterReadySubscribe;
use crate::actors::canister_http_adapter::CanisterHttpAdapter;
use crate::actors::icx_proxy::signals::PortReadySubscribe;
use crate::actors::icx_proxy::{IcxProxy, IcxProxyConfig};
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use actix::{Actor, Addr, Recipient};
use anyhow::Context;
use fn_error_context::context;
use std::fs;
use std::path::PathBuf;

pub mod btc_adapter;
pub mod canister_http_adapter;
pub mod emulator;
pub mod icx_proxy;
pub mod replica;
mod shutdown;
pub mod shutdown_controller;

#[context("Failed to start shutdown controller.")]
pub fn start_shutdown_controller(env: &dyn Environment) -> DfxResult<Addr<ShutdownController>> {
    let actor_config = shutdown_controller::Config {
        logger: Some(env.get_logger().clone()),
    };
    Ok(ShutdownController::new(actor_config).start())
}

#[context("Failed to start btc adapter.")]
pub fn start_btc_adapter_actor(
    env: &dyn Environment,
    config_path: PathBuf,
    socket_path: Option<PathBuf>,
    shutdown_controller: Addr<ShutdownController>,
    btc_adapter_pid_file_path: PathBuf,
) -> DfxResult<Addr<BtcAdapter>> {
    let btc_adapter_path = env.get_cache().get_binary_command_path("ic-btc-adapter")?;

    let actor_config = btc_adapter::Config {
        btc_adapter_path,

        config_path,
        socket_path,

        shutdown_controller,
        btc_adapter_pid_file_path,
        logger: Some(env.get_logger().clone()),
    };
    Ok(BtcAdapter::new(actor_config).start())
}

#[context("Failed to start canister http adapter actor.")]
pub fn start_canister_http_adapter_actor(
    env: &dyn Environment,
    config_path: PathBuf,
    socket_path: Option<PathBuf>,
    shutdown_controller: Addr<ShutdownController>,
    pid_file_path: PathBuf,
) -> DfxResult<Addr<CanisterHttpAdapter>> {
    let adapter_path = env
        .get_cache()
        .get_binary_command_path("ic-canister-http-adapter")?;

    let actor_config = canister_http_adapter::Config {
        adapter_path,

        config_path,
        socket_path,

        shutdown_controller,
        pid_file_path,
        logger: Some(env.get_logger().clone()),
    };
    Ok(CanisterHttpAdapter::new(actor_config).start())
}

#[context("Failed to start emulator actor.")]
pub fn start_emulator_actor(
    env: &dyn Environment,
    shutdown_controller: Addr<ShutdownController>,
    emulator_port_path: PathBuf,
) -> DfxResult<Addr<Emulator>> {
    let ic_ref_path = env.get_cache().get_binary_command_path("ic-ref")?;

    // Touch the port file. This ensures it is empty prior to
    // handing it over to ic-ref. If we read the file and it has
    // contents we shall assume it is due to our spawned ic-ref
    // process.
    std::fs::write(&emulator_port_path, "").with_context(|| {
        format!(
            "Failed to write/clear emulator port file {}.",
            emulator_port_path.to_string_lossy()
        )
    })?;

    let actor_config = actors::emulator::Config {
        ic_ref_path,
        write_port_to: emulator_port_path,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
    };
    Ok(actors::emulator::Emulator::new(actor_config).start())
}

#[context("Failed to setup replica environment.")]
fn setup_replica_env(
    local_server_descriptor: &LocalServerDescriptor,
    replica_config: &ReplicaConfig,
) -> DfxResult {
    // create replica config dir
    let replica_configuration_dir = local_server_descriptor.replica_configuration_dir();
    fs::create_dir_all(&replica_configuration_dir).with_context(|| {
        format!(
            "Failed to create replica config direcory {}.",
            replica_configuration_dir.to_string_lossy()
        )
    })?;

    if let Some(replica_port_path) = &replica_config.http_handler.write_port_to {
        // Touch the replica port file. This ensures it is empty prior to
        // handing it over to the replica. If we read the file and it has
        // contents we shall assume it is due to our spawned replica
        // process.
        std::fs::write(&replica_port_path, "").with_context(|| {
            format!(
                "Failed to write/clear replica port file {}.",
                replica_port_path.to_string_lossy()
            )
        })?;
    }

    // create replica state dir
    let state_dir = local_server_descriptor.replicated_state_dir();
    fs::create_dir_all(&state_dir).with_context(|| {
        format!(
            "Failed to create replica state directory {}.",
            state_dir.to_string_lossy()
        )
    })?;

    Ok(())
}

#[context("Failed to start replica actor.")]
pub fn start_replica_actor(
    env: &dyn Environment,
    replica_config: ReplicaConfig,
    local_server_descriptor: &LocalServerDescriptor,
    shutdown_controller: Addr<ShutdownController>,
    btc_adapter_ready_subscribe: Option<Recipient<BtcAdapterReadySubscribe>>,
    canister_http_adapter_ready_subscribe: Option<Recipient<CanisterHttpAdapterReadySubscribe>>,
) -> DfxResult<Addr<Replica>> {
    // get binary path
    let replica_path = env.get_cache().get_binary_command_path("replica")?;
    let ic_starter_path = env.get_cache().get_binary_command_path("ic-starter")?;

    setup_replica_env(local_server_descriptor, &replica_config)?;
    let replica_pid_path = local_server_descriptor.replica_pid_path();

    let actor_config = replica::Config {
        ic_starter_path,
        replica_config,
        replica_path,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
        replica_pid_path,
        btc_adapter_ready_subscribe,
        canister_http_adapter_ready_subscribe,
    };
    Ok(Replica::new(actor_config).start())
}

#[context("Failed to start icx proxy actor.")]
pub fn start_icx_proxy_actor(
    env: &dyn Environment,
    icx_proxy_config: IcxProxyConfig,
    port_ready_subscribe: Option<Recipient<PortReadySubscribe>>,
    shutdown_controller: Addr<ShutdownController>,
    icx_proxy_pid_path: PathBuf,
) -> DfxResult<Addr<IcxProxy>> {
    let icx_proxy_path = env.get_cache().get_binary_command_path("icx-proxy")?;

    let actor_config = icx_proxy::Config {
        logger: Some(env.get_logger().clone()),

        port_ready_subscribe,
        shutdown_controller,

        icx_proxy_config,
        icx_proxy_path,
        icx_proxy_pid_path,
    };
    Ok(IcxProxy::new(actor_config).start())
}
