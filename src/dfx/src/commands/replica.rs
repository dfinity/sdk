use crate::actors;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;
use crate::lib::replica_config::ReplicaConfig;

use actix::Actor;
use clap::{App, Arg, ArgMatches, SubCommand};

/// Provide necessary arguments to start the Internet Computer
/// locally. See `exec` for further information.
pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("replica")
        .about(UserMessage::Replica.to_str())
        .arg(
            Arg::with_name("port")
                .help(UserMessage::ReplicaPort.to_str())
                .long("port")
                .takes_value(true)
                .default_value("8080")
                .validator(|v| {
                    v.parse::<u16>()
                        .map_err(|_| "Must pass a valid port number.".to_owned())
                        .map(|_| ())
                }),
        )
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let state_root = env.get_temp_dir().join("state");
    let replica_pathbuf = env.get_cache().get_binary_command_path("replica")?;

    let port = args
        .value_of("port")
        .unwrap_or("8080")
        .parse::<u16>()
        .expect("Unreachable. Port should have been validated by clap.");

    let system = actix::System::new("dfx-replica");
    let mut config = ReplicaConfig::new(&state_root);
    config.with_port(port);

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
