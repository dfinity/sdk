use crate::actors;
use crate::actors::emulator::Emulator;
use crate::actors::replica::Replica;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::replica_config::ReplicaConfig;

use crate::actors::icx_proxy::signals::PortReadySubscribe;
use crate::actors::icx_proxy::{IcxProxy, IcxProxyConfig};
use actix::{Actor, Addr, Recipient};
use std::fs;
use std::path::PathBuf;

pub mod emulator;
pub mod icx_proxy;
pub mod proxy_webserver_coordinator;
pub mod replica;
mod shutdown;
pub mod shutdown_controller;

pub fn start_shutdown_controller(env: &dyn Environment) -> DfxResult<Addr<ShutdownController>> {
    let actor_config = shutdown_controller::Config {
        logger: Some(env.get_logger().clone()),
    };
    Ok(ShutdownController::new(actor_config).start())
}

pub fn start_emulator_actor(
    env: &dyn Environment,
    shutdown_controller: Addr<ShutdownController>,
) -> DfxResult<Addr<Emulator>> {
    let ic_ref_path = env.get_cache().get_binary_command_path("ic-ref")?;

    let temp_dir = env.get_temp_dir();
    let emulator_port_path = temp_dir.join("ic-ref.port");

    // Touch the port file. This ensures it is empty prior to
    // handing it over to ic-ref. If we read the file and it has
    // contents we shall assume it is due to our spawned ic-ref
    // process.
    std::fs::write(&emulator_port_path, "")?;

    let actor_config = actors::emulator::Config {
        ic_ref_path,
        write_port_to: emulator_port_path,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
    };
    Ok(actors::emulator::Emulator::new(actor_config).start())
}

fn setup_replica_env(env: &dyn Environment, replica_config: &ReplicaConfig) -> DfxResult<PathBuf> {
    // create replica config dir
    let replica_configuration_dir = env.get_temp_dir().join("replica-configuration");
    fs::create_dir_all(&replica_configuration_dir)?;

    if let Some(replica_port_path) = &replica_config.http_handler.write_port_to {
        // Touch the replica port file. This ensures it is empty prior to
        // handing it over to the replica. If we read the file and it has
        // contents we shall assume it is due to our spawned replica
        // process.
        std::fs::write(&replica_port_path, "")?;
    }

    // create replica state dir
    let state_dir = env.get_state_dir().join("replicated_state");
    fs::create_dir_all(&state_dir)?;

    Ok(replica_configuration_dir)
}

pub fn start_replica_actor(
    env: &dyn Environment,
    replica_config: ReplicaConfig,
    shutdown_controller: Addr<ShutdownController>,
) -> DfxResult<Addr<Replica>> {
    // get binary path
    let replica_path = env.get_cache().get_binary_command_path("replica")?;
    let ic_starter_path = env.get_cache().get_binary_command_path("ic-starter")?;

    let replica_configuration_dir = setup_replica_env(env, &replica_config)?;

    let actor_config = replica::Config {
        ic_starter_path,
        replica_config,
        replica_path,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
        replica_configuration_dir,
    };
    Ok(Replica::new(actor_config).start())
}

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
