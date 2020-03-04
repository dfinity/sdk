use crate::commands::canister::create_waiter;
use crate::config::dfinity::Config;
use crate::lib::client_toml_config::generate_client_configuration;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::proxy::{CoordinateProxy, ProxyConfig};
use crate::lib::proxy_process::spawn_and_update_proxy;

use clap::{App, Arg, ArgMatches, SubCommand};
use crossbeam::channel::{Receiver, Sender};
use crossbeam::unbounded;
use futures::future::Future;
use ic_http_agent::{Agent, AgentConfig};
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::fs;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use sysinfo::{Pid, Process, ProcessExt, Signal, System, SystemExt};
use tokio::runtime::Runtime;

/// Provide necessary arguments to start the Internet Computer
/// locally. See `exec` for further information.
pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("start")
        .about(UserMessage::StartNode.to_str())
        .arg(
            Arg::with_name("host")
                .help(UserMessage::NodeAddress.to_str())
                .long("host")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("background")
                .help(UserMessage::StartBackground.to_str())
                .long("background")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("clean")
                .help(UserMessage::CleanState.to_str())
                .long("clean")
                .takes_value(false),
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
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let (frontend_url, address_and_port) = frontend_address(args, &config)?;

    let client_pathbuf = env.get_cache().get_binary_command_path("replica")?;
    let temp_dir = env.get_temp_dir();
    let state_root = env.get_state_dir();

    let pid_file_path = temp_dir.join("pid");
    check_previous_process_running(&pid_file_path)?;
    // As we know no start process is running in this project, we can
    // clean up the state if it is necessary.
    if args.is_present("clean") {
        // Clean the contents of the provided directory including the
        // directory itself. N.B. This does NOT follow symbolic links -- and I
        // hope we do not need to.
        fs::remove_dir_all(state_root.clone()).map_err(DfxError::CleanState)?;
    }

    let client_configuration_dir = temp_dir.join("client-configuration");
    fs::create_dir_all(&client_configuration_dir)?;
    let client_port_path = client_configuration_dir.join("client-1.port");

    // Touch the client port file. This ensures it is empty prior to
    // handing it over to the replica. If we read the file and it has
    // contents we shall assume it is due to our spawned client
    // process.
    std::fs::write(&client_port_path, "")?;
    // We are doing this here to make sure we can write to the temp
    // pid file.
    std::fs::write(&pid_file_path, "")?;

    if args.is_present("background") {
        send_background()?;
        return ping_and_wait(&frontend_url);
    }

    // Start the client.
    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message("Starting up the client...");
    b.enable_steady_tick(80);

    // Must be unbounded, as a killed child should not deadlock.

    let (request_stop, rcv_wait) = unbounded();
    let (broadcast_stop, is_killed_client) = unbounded();
    let (give_actix, actix_handler) = unbounded();

    let request_stop_echo = request_stop.clone();
    let rcv_wait_fwatcher = rcv_wait.clone();
    b.set_message("Generating IC local replica configuration.");
    let replica_config = generate_client_configuration(&client_port_path, &state_root)?;

    // TODO(eftychis): we need a proper manager type when we start
    // spawning multiple client processes and registry.
    let client_watchdog = std::thread::Builder::new().name("replica".into()).spawn({
        let is_killed_client = is_killed_client.clone();
        let b = b.clone();

        move || {
            start_client(
                &client_pathbuf,
                &pid_file_path,
                is_killed_client,
                request_stop,
                replica_config,
                b,
            )
        }
    })?;

    let bootstrap_dir = env
        .get_cache()
        .get_binary_command_path("js-user-library/dist/bootstrap")?;

    // We have a long-lived replica process and a proxy. We use
    // currently a messaging pattern to supervise. This is going to
    // be tidied up over a more formal actor framework.
    let is_killed = is_killed_client;

    // By default we reach to no external IC nodes.
    let providers = Vec::new();

    let proxy_config = ProxyConfig {
        client_api_port: address_and_port.port(),
        bind: address_and_port,
        serve_dir: bootstrap_dir,
        providers,
    };

    let supervisor_actor_handle = CoordinateProxy {
        inform_parent: give_actix,
        server_receiver: actix_handler.clone(),
        rcv_wait_fwatcher,
        request_stop_echo,
        is_killed,
    };

    let frontend_watchdog = spawn_and_update_proxy(
        proxy_config,
        client_port_path,
        supervisor_actor_handle,
        b.clone(),
    )?;

    b.set_message("Pinging the Internet Computer client...");
    ping_and_wait(&frontend_url)?;
    b.finish_with_message("Internet Computer client started...");

    // TODO/In Progress(eftychis): Here we should define a Supervisor
    // actor to keep track and spawn these two processes
    // independently.

    // We have two side processes involving multiple threads running at
    // this point. We first wait for a signal that one of the processes
    // terminated. N.B. We do not handle the case where the proxy
    // terminates abruptly and we have to terminate the client as that
    // complicates the situation right now, and we need a watcher that
    // terminates all sibling processes if a process returns an error,
    // which we lack. We consider this a fine trade-off for now.

    rcv_wait.recv().or_else(|e| {
        Err(DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed while waiting for the manager -- {:?}", e),
        )))
    })?;

    // Signal the client to stop. Right now we have little control
    // over the client and nodemanager as it provides little
    // handling. This is mostly done for completeness. In the future
    // we should also force kill, if it ends up being necessary.
    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());
    b.set_message("Terminating...");
    b.enable_steady_tick(80);
    broadcast_stop.send(()).expect("Failed to signal children");
    // We can now start terminating our proxy server, we block to
    // ensure termination is done properly. At this point the client
    // is down though.

    // Signal the actix server to stop. This will
    // block.

    b.set_message("Terminating proxy...");
    actix_handler
        .recv()
        .expect("Failed to receive server")
        .stop(true)
        // We do not use await here on purpose. We should probably follow up
        // and have this function be async, internal of exec.
        .wait()
        .map_err(|e| {
            DfxError::RuntimeError(Error::new(
                ErrorKind::Other,
                format!("Failed to stop server: {:?}", e),
            ))
        })?;
    b.set_message("Gathering proxy thread...");
    // Join and handle errors for the frontend watchdog thread.
    frontend_watchdog.join().map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed while running frontend proxy thead -- {:?}", e),
        ))
    })?;

    b.set_message("Gathering client thread...");
    // Join and handle errors for the client watchdog thread. Here we
    // check the result of client_watchdog and start_client.
    client_watchdog.join().map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed while running client thread -- {:?}", e),
        ))
    })??;
    b.finish_with_message("Terminated successfully... Have a great day!!!");
    Ok(())
}

fn send_background() -> DfxResult<()> {
    // Background strategy is different; we spawn `dfx` with the same arguments
    // (minus --background), ping and exit.
    let exe = std::env::current_exe()?;
    let mut cmd = Command::new(exe);
    // Skip 1 because arg0 is this executable's path.
    cmd.args(std::env::args().skip(1).filter(|a| !a.eq("--background")));

    cmd.spawn()?;
    Ok(())
}

fn frontend_address(args: &ArgMatches<'_>, config: &Config) -> DfxResult<(String, SocketAddr)> {
    let address_and_port = args
        .value_of("host")
        .and_then(|host| Option::from(host.parse()))
        .unwrap_or_else(|| {
            Ok(config
                .get_config()
                .get_defaults()
                .get_start()
                .get_binding_socket_addr("localhost:8000")
                .expect("could not get socket_addr"))
        })
        .map_err(|e| DfxError::InvalidArgument(format!("Invalid host: {}", e)))?;
    let frontend_url = format!(
        "http://{}:{}",
        address_and_port.ip(),
        address_and_port.port()
    );

    Ok((frontend_url, address_and_port))
}

fn check_previous_process_running(dfx_pid_path: &PathBuf) -> DfxResult<()> {
    if dfx_pid_path.exists() {
        // Read and verify it's not running. If it is just return.
        if let Ok(s) = std::fs::read_to_string(&dfx_pid_path) {
            if let Ok(pid) = s.parse::<i32>() {
                // If we find the pid in the file, we tell the user and don't start!
                let system = System::new();
                if let Some(_process) = system.get_process(pid) {
                    return Err(DfxError::DfxAlreadyRunningInBackground());
                }
            }
        }
    }
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
