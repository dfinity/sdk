use crate::commands::canister::create_waiter;
use crate::config::dfinity::ConfigDefaultsReplica;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::replica_config::{
    HttpHandlerConfig, ReplicaConfig, SchedulerConfig, StateManagerConfig,
};

use clap::{App, Arg, ArgMatches, SubCommand};
use crossbeam::channel::{Receiver, Sender};
use crossbeam::unbounded;
use ic_http_agent::{Agent, AgentConfig};
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::default::Default;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::time::Duration;
use sysinfo::{Pid, Process, ProcessExt, Signal};
use tokio::runtime::Runtime;

/// Constructs a sub-command to run the Internet Computer replica.
pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("replica")
        .about(UserMessage::Replica.to_str())
        .arg(
            Arg::with_name("message-gas-limit")
                .help(UserMessage::ReplicaMessageGasLimit.to_str())
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
    let state_manager = StateManagerConfig {
        state_root: env.get_state_dir(),
    };
    Ok(ReplicaConfig {
        http_handler: http_handler,
        scheduler: scheduler,
        state_manager: state_manager,
    })
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
            let default = 5368709120;
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
            let default = 26843545600;
            Ok(config.round_gas_limit.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid round gas limit: {}", err)))
}

fn ping_and_wait(frontend_url: &str) -> DfxResult {
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let agent = Agent::new(AgentConfig {
        url: frontend_url,
        ..AgentConfig::default()
    })?;
    runtime
        .block_on(agent.ping(create_waiter()))
        .map_err(DfxError::from)
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let replica_binary_path = env.get_cache().get_binary_command_path("replica")?;
    let temp_dir = env.get_temp_dir();
    let pid_file_path = temp_dir.join("pid");

    // We are doing this here to make sure we can write to the temp
    // pid file.
    std::fs::write(&pid_file_path, "")?;

    // Start the replica.
    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message("Starting up the replica...");
    b.enable_steady_tick(80);

    // Must be unbounded, as a killed child should not deadlock.
    let (request_stop, _rcv_wait) = unbounded();
    let (_broadcast_stop, is_killed_replica) = unbounded();

    b.set_message("Generating IC local replica configuration.");
    let config = get_config(env, args)?;
    let port = config.http_handler.use_port.expect("non-random port");
    let toml = config.to_toml()?;

    // TODO(eftychis): we need a proper manager type when we start
    // spawning multiple replica processes and registry.
    let replica_watchdog = std::thread::Builder::new().name("replica".into()).spawn({
        let b = b.clone();

        move || {
            start_replica(
                &replica_binary_path,
                &pid_file_path,
                is_killed_replica,
                request_stop,
                toml,
                b,
            )
        }
    })?;

    b.set_message("Pinging the Internet Computer replica...");
    ping_and_wait(format!("http://localhost:{}", port).as_str())?;
    b.finish_with_message("Internet Computer replica started...");

    // Join and handle errors for the replica watchdog thread. Here we
    // check the result of replica_watchdog and start_replica.
    replica_watchdog.join().map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed while running replica thread -- {:?}", e),
        ))
    })??;

    Ok(())
}

/// Starts the replica. It is supposed to be used in a thread, thus
/// this function will panic when an error occurs that implies
/// termination of the replica and need the attention of the parent
/// thread.
///
/// # Panics
/// We panic here to transmit an error to the parent thread.
fn start_replica(
    replica_pathbuf: &PathBuf,
    pid_file_path: &PathBuf,
    is_killed_replica: Receiver<()>,
    request_stop: Sender<()>,
    config: String,
    b: ProgressBar,
) -> DfxResult<()> {
    b.set_message("Generating IC local replica configuration.");
    let replica = replica_pathbuf.as_path().as_os_str();

    let mut cmd = std::process::Command::new(replica);
    cmd.args(&["--config", config.as_str()]);
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    // If the replica itself fails, we are probably into deeper trouble than
    // we can solve at this point and the user is better rerunning the server.
    let mut child = cmd.spawn().unwrap_or_else(|e| {
        request_stop
            .try_send(())
            .expect("Replica thread couldn't signal parent to stop");
        // We still want to send an error message.
        panic!("Couldn't spawn node manager with command {:?}: {}", cmd, e);
    });

    // Update the pid file.
    if let Ok(pid) = sysinfo::get_current_pid() {
        let _ = std::fs::write(&pid_file_path, pid.to_string());
    }

    // N.B. The logic below fixes errors from replica causing
    // restarts. We do not want to respawn the replica on a failure.
    // This should be substituted with a supervisor.

    // Did we receive a kill signal?
    while is_killed_replica.is_empty() {
        // We have to wait for the child to exit here. We *should*
        // always wait(). Read related documentation.

        // We check every 1s on the replica. This logic should be
        // transferred / substituted by a supervisor object.
        std::thread::sleep(Duration::from_millis(1000));

        match child.try_wait() {
            Ok(Some(status)) => {
                // An error occurred: exit the loop.
                b.set_message(format!("local replica exited with: {}", status).as_str());
                break;
            }
            Ok(None) => {
                // No change in exit status.
                continue;
            }
            Err(e) => {
                request_stop
                    .send(())
                    .expect("Could not signal parent thread from replica runner");
                panic!("Failed to check the status of the replica: {}", e)
            }
        }
    }
    // Terminate the replica; wait() then signal stop. Ignore errors
    // -- we might get InvalidInput: that is fine -- process might
    // have terminated already.
    Process::new(child.id() as Pid, None, 0).kill(Signal::Term);
    match child.wait() {
        Ok(status) => b.set_message(format!("Replica exited with {}", status).as_str()),
        Err(e) => b.set_message(
            format!("Failed to properly wait for the replica to terminate {}", e).as_str(),
        ),
    }

    // We DO want to panic here, if we can not signal our
    // parent. This is interpreted as an error via join by the
    // parent thread.
    request_stop
        .send(())
        .expect("Could not signal parent thread from replica runner");
    Ok(())
}
