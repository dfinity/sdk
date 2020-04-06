use crate::actors;
use crate::config::dfinity::ConfigDefaultsReplica;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::replica_config::{HttpHandlerConfig, ReplicaConfig, SchedulerConfig};

use actix::Actor;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::default::Default;

/// Constructs a sub-command to run the Internet Computer replica.
pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("replica")
        .about(UserMessage::Replica.to_str())
        .arg(
            Arg::with_name("message-gas-limit")
                .help(UserMessage::ReplicaMessageGasLimit.to_str())
                .hidden(true)
                .long("message-gas-limit")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .help(UserMessage::ReplicaPort.to_str())
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("round-gas-limit")
                .help(UserMessage::ReplicaRoundGasLimit.to_str())
                .hidden(true)
                .long("round-gas-limit")
                .takes_value(true),
        )
}

/// Gets the configuration options for the Internet Computer replica.
fn get_config(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult<ReplicaConfig> {
    let config = get_config_from_file(env);
    let port = get_port(&config, args)?;
    let mut http_handler: HttpHandlerConfig = Default::default();
    if port == 0 {
        let file = env.get_temp_dir().join("config").join("port.txt");
        http_handler.write_port_to = Some(file);
    } else {
        http_handler.use_port = Some(port);
    };
    let message_gas_limit = get_message_gas_limit(&config, args)?;
    let round_gas_limit = get_round_gas_limit(&config, args)?;
    let scheduler = SchedulerConfig {
        exec_gas: Some(message_gas_limit),
        round_gas_max: Some(round_gas_limit),
    }
    .validate()?;

    let mut replica_config = ReplicaConfig::new(env.get_state_dir());
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
fn get_port(config: &ConfigDefaultsReplica, args: &ArgMatches<'_>) -> DfxResult<u16> {
    args.value_of("port")
        .map(|port| port.parse())
        .unwrap_or_else(|| {
            let default = 8080;
            Ok(config.port.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid port number: {}", err)))
}

/// Gets the maximum amount of gas a single message can consume. First checks if the gas limit was
/// specified on the command-line using --message-gas-limit, otherwise checks if the gas limit was
/// specified in the dfx configuration file, otherise defaults to 5368709120.
fn get_message_gas_limit(config: &ConfigDefaultsReplica, args: &ArgMatches<'_>) -> DfxResult<u64> {
    args.value_of("message-gas-limit")
        .map(|limit| limit.parse())
        .unwrap_or_else(|| {
            let default = 5_368_709_120;
            Ok(config.message_gas_limit.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid message gas limit: {}", err)))
}

/// Gets the maximum amount of gas a single round can consume. First checks if the gas limit was
/// specified on the command-line using --round-gas-limit, otherwise checks if the gas limit was
/// specified in the dfx configuration file, otherise defaults to 26843545600.
fn get_round_gas_limit(config: &ConfigDefaultsReplica, args: &ArgMatches<'_>) -> DfxResult<u64> {
    args.value_of("round-gas-limit")
        .map(|limit| limit.parse())
        .unwrap_or_else(|| {
            let default = 26_843_545_600;
            Ok(config.round_gas_limit.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid round gas limit: {}", err)))
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let replica_pathbuf = env.get_cache().get_binary_command_path("replica")?;

    let system = actix::System::new("dfx-replica");
    let config = get_config(env, args)?;

    actors::replica::Replica::new(actors::replica::Config {
        replica_config: config,
        replica_path: replica_pathbuf,
        logger: Some(env.get_logger().clone()),
    })
    .start();

    actors::signal_watcher::SignalWatchdog::new().start();
    system.run()?;

    Ok(())
}
