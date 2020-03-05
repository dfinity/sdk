use crate::commands::canister::create_waiter;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::replica_config::ReplicaConfig;

use clap::{App, Arg, ArgMatches, SubCommand};
use crossbeam::channel::{Receiver, Sender};
use crossbeam::unbounded;
use ic_http_agent::{Agent, AgentConfig};
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::time::Duration;
use sysinfo::{Pid, Process, ProcessExt, Signal};
use tokio::runtime::Runtime;

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

// TODO(eftychis)/In progress: Rename to replica.
/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let replica_binary_path = env.get_cache().get_binary_command_path("replica")?;
    let temp_dir = env.get_temp_dir();
    let state_root = env.get_state_dir();
    let pid_file_path = temp_dir.join("pid");

    let port = args
        .value_of("port")
        .unwrap_or("8080")
        .parse::<u16>()
        .expect("Unreachable. Port should have been validated by clap.");

    // We are doing this here to make sure we can write to the temp
    // pid file.
    std::fs::write(&pid_file_path, "")?;

    // Start the client.
    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message("Starting up the client...");
    b.enable_steady_tick(80);

    // Must be unbounded, as a killed child should not deadlock.
    let (request_stop, _rcv_wait) = unbounded();
    let (_broadcast_stop, is_killed_client) = unbounded();

    b.set_message("Generating IC local replica configuration.");
    let replica_config = ReplicaConfig::new(&state_root).with_port(port).to_toml()?;

    // TODO(eftychis): we need a proper manager type when we start
    // spawning multiple client processes and registry.
    let client_watchdog = std::thread::Builder::new().name("replica".into()).spawn({
        let is_killed_client = is_killed_client.clone();
        let b = b.clone();

        move || {
            start_client(
                &replica_binary_path,
                &pid_file_path,
                is_killed_client,
                request_stop,
                replica_config,
                b,
            )
        }
    })?;

    b.set_message("Pinging the Internet Computer client...");
    ping_and_wait(format!("http://localhost:{}", port).as_str())?;
    b.finish_with_message("Internet Computer client started...");

    // Join and handle errors for the client watchdog thread. Here we
    // check the result of client_watchdog and start_client.
    client_watchdog.join().map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed while running client thread -- {:?}", e),
        ))
    })??;

    Ok(())
}

/// Starts the client. It is supposed to be used in a thread, thus
/// this function will panic when an error occurs that implies
/// termination of the replica and need the attention of the parent
/// thread.
///
/// # Panics
/// We panic here to transmit an error to the parent thread.
fn start_client(
    client_pathbuf: &PathBuf,
    pid_file_path: &PathBuf,
    is_killed_client: Receiver<()>,
    request_stop: Sender<()>,
    config: String,
    b: ProgressBar,
) -> DfxResult<()> {
    b.set_message("Generating IC local replica configuration.");
    let client = client_pathbuf.as_path().as_os_str();

    let mut cmd = std::process::Command::new(client);
    cmd.args(&["--config", config.as_str()]);
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    // If the replica itself fails, we are probably into deeper trouble than
    // we can solve at this point and the user is better rerunning the server.
    let mut child = cmd.spawn().unwrap_or_else(|e| {
        request_stop
            .try_send(())
            .expect("Client thread couldn't signal parent to stop");
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
    while is_killed_client.is_empty() {
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
                    .expect("Could not signal parent thread from client runner");
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
        .expect("Could not signal parent thread from client runner");
    Ok(())
}
