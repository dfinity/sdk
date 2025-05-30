use self::pocketic::PocketIc;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::progress_bar::ProgressBar;
use actix::{Actor, Addr, Recipient};
use anyhow::Context;
use dfx_core::config::model::local_server_descriptor::LocalServerDescriptor;
use dfx_core::config::model::replica_config::ReplicaConfig;
use fn_error_context::context;
use pocketic::BitcoinIntegrationConfig;
use pocketic_proxy::signals::PortReadySubscribe;
use pocketic_proxy::{PocketIcProxy, PocketIcProxyConfig};
use post_start::PostStart;
use std::path::PathBuf;

pub mod pocketic;
pub mod pocketic_proxy;
pub mod post_start;
mod shutdown;
pub mod shutdown_controller;

#[context("Failed to start shutdown controller.")]
pub fn start_shutdown_controller(env: &dyn Environment) -> DfxResult<Addr<ShutdownController>> {
    let actor_config = shutdown_controller::Config {
        logger: Some(env.get_logger().clone()),
    };
    Ok(ShutdownController::new(actor_config).start())
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
    let pocketic_proxy_path = env.get_cache().get_binary_command_path(env, "pocket-ic")?;
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
    let pocketic_path = env.get_cache().get_binary_command_path(env, "pocket-ic")?;

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

    let bitcoin_integration_config = if local_server_descriptor.bitcoin.enabled {
        Some(BitcoinIntegrationConfig {
            canister_init_arg: local_server_descriptor.bitcoin.canister_init_arg.clone(),
        })
    } else {
        None
    };
    let actor_config = pocketic::Config {
        pocketic_path,
        effective_config_path: local_server_descriptor.effective_config_path(),
        replica_config,
        bitcoind_addr: local_server_descriptor.bitcoin.nodes.clone(),
        bitcoin_integration_config,
        port: local_server_descriptor.replica.port,
        port_file: pocketic_port_path,
        pid_file: local_server_descriptor.pocketic_pid_path(),
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
    };
    Ok(pocketic::PocketIc::new(actor_config).start())
}

#[context("Failed to start PostStart actor.")]
pub fn start_post_start_actor(
    env: &dyn Environment,
    background: bool,
    pocketic_proxy: Option<Addr<PocketIcProxy>>,
    spinner: ProgressBar,
) -> DfxResult<Addr<PostStart>> {
    let config = post_start::Config {
        logger: env.get_logger().clone(),
        background,
        pocketic_proxy,
    };
    Ok(PostStart::new(config, spinner).start())
}
