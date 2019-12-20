use crate::config::dfinity::Config;
use crate::lib::api_client::{ping, Client, ClientConfig};
use crate::lib::env::{BinaryResolverEnv, ProjectConfigEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::webserver::webserver;
use clap::{App, Arg, ArgMatches, SubCommand};
use crossbeam::channel::{Receiver, Sender};
use crossbeam::unbounded;
use futures::future::Future;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use indicatif::{ProgressBar, ProgressDrawTarget};
use serde::Serialize;
use std::fs;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt};
use tokio::prelude::FutureExt;
use tokio::runtime::Runtime;

const TIMEOUT_IN_SECS: u64 = 10;
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
}

fn ping_and_wait(frontend_url: &str) -> DfxResult {
    std::thread::sleep(Duration::from_millis(500));

    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    // Try to ping for 1 second, then timeout after 5 seconds if ping hasn't succeeded.
    let start = Instant::now();
    while {
        let client = Client::new(ClientConfig {
            url: frontend_url.to_string(),
        });

        runtime
            .block_on(ping(client).timeout(Duration::from_millis(300)))
            .is_err()
    } {
        if Instant::now().duration_since(start) > Duration::from_secs(TIMEOUT_IN_SECS) {
            return Err(DfxError::Unknown(
                "Timeout during start of the client.".to_owned(),
            ));
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    Ok(())
}

// TODO: Refactor exec into more manageable pieces.
pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: ProjectConfigEnv + BinaryResolverEnv,
{
    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let (frontend_url, address_and_port) = frontend_address(args, config)?;

    let client_pathbuf = env.get_binary_command_path("client")?;
    let nodemanager_pathbuf = env.get_binary_command_path("nodemanager")?;

    let project_root = config.get_path().parent().unwrap();
    let pid_file_path = env.get_dfx_root().unwrap().join("pid");

    check_previous_process_running(&pid_file_path)?;

    let client_configuration_dir = env.get_dfx_root().unwrap().join("client-configuration");
    fs::create_dir_all(&client_configuration_dir)?;
    let client_configuration_path = client_configuration_dir.join("client-1.toml");
    fs::File::create(&client_configuration_path)?;
    let client_port_path = client_configuration_dir.join("client-1.port");

    place_client_configuration(&client_configuration_path, &client_port_path)?;
    // Touch the client port file. But ensure it is empty prior to
    // that. This ensures if we read the file and it has contents we
    // can assume it is due to our spawned client process.
    std::fs::write(&client_port_path, "")?;

    // We are doing this here to make sure we can write to the temp pid file.
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

    // We wait for the port to be determined here. Note this sets the
    // stage for a few of issues:
    // i) What happens if the port file is moved? (Undefined behaviour)
    // ii) How do we deal with client failures, as we now block
    // iii) What if another process modifies the file? (ignore)
    // iv) order of execution between watcher and client

    let watcher = std::thread::Builder::new()
        .name("File Watcher".into())
        .spawn({
            let b = b.clone();
            move || {
                retrieve_client_port(&client_port_path, rcv_wait_fwatcher, request_stop_echo, &b)
            }
        })?;

    // Ensure watcher is ready. Poor man's solution to keep things
    // sane.
    // TODO(eftychis): Restructure this with the client
    // refactoring, which should make this not necessary.
    std::thread::sleep(Duration::from_millis(20));

    // TODO(eftychis): we need a proper manager type when we start
    // spawning multiple client processes and registry.
    let client_watchdog = std::thread::Builder::new()
        .name("NodeManager".into())
        .spawn(move || {
            start_client(
                &client_pathbuf.clone(),
                &nodemanager_pathbuf,
                &pid_file_path,
                is_killed_client,
                request_stop,
                &client_configuration_path,
            )
        })?;

    // Now we can read the file. If there are no contents we need to
    // fail. We check if the watcher thinks the file has been written.
    let client_port: String = watcher.join().map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed while running frontend proxy thead -- {:?}", e),
        ))
    })??;
    eprintln!("Client bound at {}", client_port);
    let ic_client_bind_addr = "http://localhost:".to_owned() + client_port.as_str() + "/api";
    let ic_client_bind_addr = ic_client_bind_addr.as_str();

    let frontend_watchdog = webserver(
        address_and_port,
        url::Url::parse(ic_client_bind_addr).unwrap(),
        project_root
            .join(
                config
                    .get_config()
                    .get_defaults()
                    .get_start()
                    .get_serve_root(".")
                    .as_path(),
            )
            .as_path(),
        give_actix,
    );

    b.set_message("Pinging the Internet Computer client...");
    ping_and_wait(&frontend_url)?;
    b.finish_with_message("Internet Computer client started...");

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

fn start_client(
    client_pathbuf: &PathBuf,
    nodemanager_pathbuf: &PathBuf,
    pid_file_path: &PathBuf,
    is_killed_client: Receiver<()>,
    request_stop: Sender<()>,
    config_path: &PathBuf,
) -> DfxResult<()> {
    let client = client_pathbuf.as_path();
    let nodemanager = nodemanager_pathbuf.as_path();
    // We use unwrap() here to transmit an error to the parent
    // thread.
    while is_killed_client.is_empty() {
        let mut cmd = std::process::Command::new(nodemanager);
        cmd.args(&[client, config_path]);
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        // If the nodemanager itself fails, we are probably deeper into troubles than
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

        // We have to wait for the child to exit here. We *should*
        // always wait(). Read related documentation.
        if child.wait().is_err() {
            break;
        }
    }
    // We DO want to panic here, if we can not signal our
    // parent. This is interpreted as an error via join by the
    // parent thread.
    request_stop
        .send(())
        .expect("Could not signal parent thread from client runner");
    Ok(())
}

fn place_client_configuration(configuration_path: &PathBuf, port_file_path: &PathBuf) -> DfxResult {
    let config = generate_client_configuration(port_file_path)?;
    eprintln!(
        "Writing client configuration file to: {:?}",
        configuration_path
    );
    fs::write(configuration_path, config).map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed to write file: {:?}", e),
        ))
    })
}

#[derive(Debug, Serialize)]
struct HttpHandlerConfig<'a> {
    write_port_to: &'a PathBuf,
}
#[derive(Debug, Serialize)]
struct ClientTomlConfig<'a> {
    http_handler: HttpHandlerConfig<'a>,
}

fn generate_client_configuration(port_file_path: &PathBuf) -> DfxResult<String> {
    let http_values = ClientTomlConfig {
        http_handler: HttpHandlerConfig {
            write_port_to: port_file_path,
        },
    };
    toml::to_string(&http_values).map_err(DfxError::CouldNotSerializeClientConfiguration)
}

fn retrieve_client_port(
    client_port_path: &PathBuf,
    rcv_wait_fwatcher: Receiver<()>,
    request_stop_echo: Sender<()>,
    b: &ProgressBar,
) -> DfxResult<String> {
    let mut watcher = Hotwatch::new_with_custom_delay(Duration::from_millis(1)).map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed to create watcher for port pid file: {}", e),
        ))
    })?;

    watcher
        .watch(&client_port_path, move |event| {
            if let Ok(e) = rcv_wait_fwatcher.try_recv() {
                // We are in a weird state where the nodemanager exited with an error,
                // but we are still waiting for the pid file to change. As this change
                // is never going to occur we need to exit our wait and stop tracking
                // the file. We need to re-send the error to properly handle it later
                // on. Worst case we will panic at this point.
                #[allow(clippy::unit_arg)]
                request_stop_echo
                    // We are re-sending the signal here. It is a unit
                    // right now but that can easily change.
                    .send(e)
                    .expect("Watcher could not re-signal request to stop.");
                return Flow::Exit;
            }
            match event {
                // We pretty much want to unblock for any events
                // except a rescan. A move, create etc event should
                // lead to a failure.
                Event::Rescan => Flow::Continue,
                _ => Flow::Exit,
            }
        })
        .map_err(|e| {
            DfxError::RuntimeError(Error::new(
                ErrorKind::Other,
                format!("Failed to watch port pid file: {}", e),
            ))
        })?;
    b.set_message("Waiting for client to bind their http server port...");
    // We are blocking here and actually processing write events.
    watcher.run();
    fs::read_to_string(&client_port_path).map_err(DfxError::RuntimeError)
}
