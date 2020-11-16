#[macro_use]
use crate::{error_invalid_argument};
use crate::actors;
use crate::actors::shutdown_controller;
use crate::actors::shutdown_controller::ShutdownController;
use crate::config::dfinity::ConfigDefaultsReplica;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::replica_config::{HttpHandlerConfig, ReplicaConfig, SchedulerConfig};
use actix::Actor;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use std::default::Default;

/// Starts a local Internet Computer replica.
#[derive(Clap)]
#[clap(name("replica"))]
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
}

pub fn construct() -> App<'static> {
    ReplicaOpts::into_app()
}

/// Gets the configuration options for the Internet Computer replica.
fn get_config(env: &dyn Environment, opts: ReplicaOpts) -> DfxResult<ReplicaConfig> {
    let config = get_config_from_file(env);
    let port = get_port(&config, opts.port)?;
    let mut http_handler: HttpHandlerConfig = Default::default();
    if port == 0 {
        let config_dir = env.get_temp_dir().join("config");
        std::fs::create_dir_all(&config_dir)?;
        let file = config_dir.join("port.txt");
        http_handler.write_port_to = Some(file);
    } else {
        http_handler.port = Some(port);
    };
    let message_gas_limit = get_message_gas_limit(&config, opts.message_gas_limit)?;
    let round_gas_limit = get_round_gas_limit(&config, opts.round_gas_limit)?;
    let scheduler = SchedulerConfig {
        exec_gas: Some(message_gas_limit),
        round_gas_max: Some(round_gas_limit),
    }
    .validate()?;

    let temp_dir = env.get_temp_dir();
    let state_dir = temp_dir.join("state/replicated_state");
    std::fs::create_dir_all(&state_dir)?;

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

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: ReplicaOpts = ReplicaOpts::from_arg_matches(args);
    let replica_pathbuf = env.get_cache().get_binary_command_path("replica")?;
    let ic_starter_pathbuf = env.get_cache().get_binary_command_path("ic-starter")?;

    let system = actix::System::new("dfx-replica");
    let config = get_config(env, opts)?;

    let shutdown_controller = ShutdownController::new(shutdown_controller::Config {
        logger: Some(env.get_logger().clone()),
    })
    .start();

    let _replica_addr = actors::replica::Replica::new(actors::replica::Config {
        ic_starter_path: ic_starter_pathbuf,
        replica_config: config,
        replica_path: replica_pathbuf,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
    })
    .start();

    system.run()?;

    Ok(())
}
