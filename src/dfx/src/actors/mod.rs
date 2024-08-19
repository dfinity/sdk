use crate::actors::btc_adapter::signals::BtcAdapterReadySubscribe;
use crate::actors::btc_adapter::BtcAdapter;
use crate::actors::canister_http_adapter::signals::CanisterHttpAdapterReadySubscribe;
use crate::actors::canister_http_adapter::CanisterHttpAdapter;
use crate::actors::replica::{BitcoinIntegrationConfig, Replica};
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use actix::{Actor, Addr, Recipient};
use anyhow::Context;
use dfx_core::config::model::local_server_descriptor::LocalServerDescriptor;
use dfx_core::config::model::replica_config::ReplicaConfig;
use fn_error_context::context;
use pocketic_proxy::signals::PortReadySubscribe;
use pocketic_proxy::{PocketIcProxy, PocketIcProxyConfig};
use std::fs;
use std::path::PathBuf;

use self::pocketic::PocketIc;

pub mod btc_adapter;
pub mod canister_http_adapter;
pub mod pocketic;
pub mod pocketic_proxy;
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
) -> DfxResult<Recipient<BtcAdapterReadySubscribe>> {
    let btc_adapter_path = env.get_cache().get_binary_command_path("ic-btc-adapter")?;

    let actor_config = btc_adapter::Config {
        btc_adapter_path,

        config_path,
        socket_path,

        shutdown_controller,
        btc_adapter_pid_file_path,
        logger: Some(env.get_logger().clone()),
    };
    Ok(BtcAdapter::new(actor_config).start().recipient())
}

#[context("Failed to start canister http adapter actor.")]
pub fn start_canister_http_adapter_actor(
    env: &dyn Environment,
    config_path: PathBuf,
    socket_path: Option<PathBuf>,
    shutdown_controller: Addr<ShutdownController>,
    pid_file_path: PathBuf,
) -> DfxResult<Recipient<CanisterHttpAdapterReadySubscribe>> {
    let adapter_path = env
        .get_cache()
        .get_binary_command_path("ic-https-outcalls-adapter")?;

    let actor_config = canister_http_adapter::Config {
        adapter_path,

        config_path,
        socket_path,

        shutdown_controller,
        pid_file_path,
        logger: Some(env.get_logger().clone()),
    };
    Ok(CanisterHttpAdapter::new(actor_config).start().recipient())
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
            "Failed to create replica config directory {}.",
            replica_configuration_dir.to_string_lossy()
        )
    })?;

    if let Some(replica_port_path) = &replica_config.http_handler.write_port_to {
        // Touch the replica port file. This ensures it is empty prior to
        // handing it over to the replica. If we read the file and it has
        // contents we shall assume it is due to our spawned replica
        // process.
        std::fs::write(replica_port_path, "").with_context(|| {
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

    let bitcoin_integration_config = if local_server_descriptor.bitcoin.enabled {
        let canister_init_arg = local_server_descriptor.bitcoin.canister_init_arg.clone();
        Some(BitcoinIntegrationConfig { canister_init_arg })
    } else {
        None
    };

    let actor_config = replica::Config {
        ic_starter_path,
        replica_config,
        bitcoin_integration_config,
        replica_path,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
        replica_pid_path,
        btc_adapter_ready_subscribe,
        canister_http_adapter_ready_subscribe,
    };
    Ok(Replica::new(actor_config).start())
}

#[context("Failed to start HTTP gateway actor.")]
pub fn start_pocketic_proxy_actor(
    env: &dyn Environment,
    pocketic_proxy_config: PocketIcProxyConfig,
    port_ready_subscribe: Option<Recipient<PortReadySubscribe>>,
    shutdown_controller: Addr<ShutdownController>,
    pocketic_proxy_pid_path: PathBuf,
    pocketic_proxy_port_path: PathBuf,
) -> DfxResult<Addr<PocketIcProxy>> {
    let pocketic_proxy_path = env.get_cache().get_binary_command_path("pocket-ic")?;
    let actor_config = pocketic_proxy::Config {
        logger: Some(env.get_logger().clone()),
        port_ready_subscribe,
        shutdown_controller,
        pocketic_proxy_config,
        pocketic_proxy_path,
        pocketic_proxy_pid_path,
        pocketic_proxy_port_path,
    };
    Ok(PocketIcProxy::new(actor_config).start())
}

#[context("Failed to start PocketIC actor.")]
pub fn start_pocketic_actor(
    env: &dyn Environment,
    replica_config: ReplicaConfig,
    local_server_descriptor: &LocalServerDescriptor,
    shutdown_controller: Addr<ShutdownController>,
    pocketic_port_path: PathBuf,
) -> DfxResult<Addr<PocketIc>> {
    let pocketic_path = env.get_cache().get_binary_command_path("pocket-ic")?;

    // Touch the port file. This ensures it is empty prior to
    // handing it over to PocketIC. If we read the file and it has
    // contents we shall assume it is due to our spawned pocket-ic
    // process.
    std::fs::write(&pocketic_port_path, "").with_context(|| {
        format!(
            "Failed to write/clear PocketIC port file {}.",
            pocketic_port_path.to_string_lossy()
        )
    })?;

    let actor_config = pocketic::Config {
        pocketic_path,
        replica_config,
        port: local_server_descriptor.replica.port,
        port_file: pocketic_port_path,
        pid_file: local_server_descriptor.pocketic_pid_path(),
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
        verbose: env.get_verbose_level() > 0,
    };
    Ok(pocketic::PocketIc::new(actor_config).start())
}
