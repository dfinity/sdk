use crate::actors::shutdown_controller::ShutdownController;
use crate::actors::{start_emulator_actor, start_replica_actor, start_shutdown_controller};
use crate::config::dfinity::ConfigDefaultsReplica;
use crate::error_invalid_argument;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::replica_config::{HttpHandlerConfig, ReplicaConfig, SchedulerConfig};

use actix::Addr;
use clap::Clap;
use std::default::Default;

/// Starts a local Internet Computer replica.
#[derive(Clap)]
pub struct ReplicaOpts {
    /// Specifies the maximum number of cycles a single message can consume.
    #[clap(long, hidden = true)]
    message_gas_limit: Option<String>,

    /// Specifies the port the local replica should listen to.
    #[clap(long)]
    port: Option<String>,

    /// Specifies the maximum number of cycles a single round can consume.
    #[clap(long, hidden = true)]
    round_gas_limit: Option<String>,

    /// Runs a dedicated emulator instead of the replica
    #[clap(long)]
    emulator: bool,
}

/// Gets the configuration options for the Internet Computer replica.
fn get_config(env: &dyn Environment, opts: ReplicaOpts) -> DfxResult<ReplicaConfig> {
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

    // get replica command opts
    let message_gas_limit = get_message_gas_limit(&config, opts.message_gas_limit)?;
    let round_gas_limit = get_round_gas_limit(&config, opts.round_gas_limit)?;
    let scheduler = SchedulerConfig {
        exec_gas: Some(message_gas_limit),
        round_gas_max: Some(round_gas_limit),
    }
    .validate()?;

    let mut replica_config = ReplicaConfig::new(&env.get_state_dir());
    replica_config.http_handler = http_handler;
    replica_config.scheduler = scheduler;
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
fn get_port(config: &ConfigDefaultsReplica, port: Option<String>) -> DfxResult<u16> {
    port.map(|port| port.parse())
        .unwrap_or_else(|| {
            let default = 8080;
            Ok(config.port.unwrap_or(default))
        })
        .map_err(|err| error_invalid_argument!("Invalid port number: {}", err))
}

/// Gets the maximum amount of gas a single message can consume. First checks if the gas limit was
/// specified on the command-line using --message-gas-limit, otherwise checks if the gas limit was
/// specified in the dfx configuration file, otherise defaults to 5368709120.
fn get_message_gas_limit(
    config: &ConfigDefaultsReplica,
    message_gas_limit: Option<String>,
) -> DfxResult<u64> {
    message_gas_limit
        .map(|limit| limit.parse())
        .unwrap_or_else(|| {
            let default = 5_368_709_120;
            Ok(config.message_gas_limit.unwrap_or(default))
        })
        .map_err(|err| error_invalid_argument!("Invalid message gas limit: {}", err))
}

/// Gets the maximum amount of gas a single round can consume. First checks if the gas limit was
/// specified on the command-line using --round-gas-limit, otherwise checks if the gas limit was
/// specified in the dfx configuration file, otherise defaults to 26843545600.
fn get_round_gas_limit(
    config: &ConfigDefaultsReplica,
    round_gas_limit: Option<String>,
) -> DfxResult<u64> {
    round_gas_limit
        .map(|limit| limit.parse())
        .unwrap_or_else(|| {
            let default = 26_843_545_600;
            Ok(config.round_gas_limit.unwrap_or(default))
        })
        .map_err(|err| error_invalid_argument!("Invalid round gas limit: {}", err))
}

fn start_replica(
    env: &dyn Environment,
    opts: ReplicaOpts,
    shutdown_controller: Addr<ShutdownController>,
) -> DfxResult {
    let replica_config = get_config(env, opts)?;
    start_replica_actor(env, replica_config, shutdown_controller)?;
    Ok(())
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, opts: ReplicaOpts) -> DfxResult {
    let system = actix::System::new("dfx-replica");
    let shutdown_controller = start_shutdown_controller(env)?;
    if opts.emulator {
        start_emulator_actor(env, shutdown_controller)?;
    } else {
        start_replica(env, opts, shutdown_controller)?;
    }
    system.run()?;
    Ok(())
}
