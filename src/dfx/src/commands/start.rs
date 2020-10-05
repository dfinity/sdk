use crate::config::dfinity::Config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::provider::get_network_descriptor;
use crate::lib::replica_config::ReplicaConfig;
use crate::util::get_reusable_socket_addr;

use crate::actors;
use crate::actors::replica::Replica;
use crate::actors::replica_webserver_coordinator::ReplicaWebserverCoordinator;
use crate::actors::shutdown_controller;
use crate::actors::shutdown_controller::ShutdownController;
use actix::{Actor, Addr};
use clap::{App, Arg, ArgMatches, SubCommand};
use delay::{Delay, Waiter};
use ic_agent::Agent;
use std::fs;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use sysinfo::{System, SystemExt};
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

    let agent = Agent::builder().with_url(frontend_url).build()?;

    // wait for frontend to come up
    let mut waiter = Delay::builder()
        .timeout(std::time::Duration::from_secs(30))
        .throttle(std::time::Duration::from_secs(1))
        .build();

    runtime.block_on(async {
        waiter.start();
        loop {
            let status = agent.status().await;
            if status.is_ok() {
                break;
            }
            waiter
                .wait()
                .map_err(|_| DfxError::AgentError(status.unwrap_err()))?;
        }
        Ok(())
    })
}

// The frontend webserver is brought up by the bg process; thus, the fg process
// needs to wait and verify it's up before exiting.
// Because the user may have specified to start on port 0, here we wait for
// webserver_port_path to get written to and modify the frontend_url so we
// ping the correct address.
fn fg_ping_and_wait(webserver_port_path: PathBuf, frontend_url: String) -> DfxResult {
    let mut waiter = Delay::builder()
        .timeout(std::time::Duration::from_secs(30))
        .throttle(std::time::Duration::from_secs(1))
        .build();
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let port = runtime.block_on(async {
        waiter.start();
        let mut contents = String::new();
        loop {
            let tokio_file = tokio::fs::File::open(&webserver_port_path).await?;
            let mut std_file = tokio_file.into_std().await;
            std_file.read_to_string(&mut contents)?;
            if !contents.is_empty() {
                break;
            }
            waiter.wait()?;
        }
        Ok::<String, DfxError>(contents.clone())
    })?;
    let mut frontend_url_mod = frontend_url.clone();
    let port_offset = frontend_url_mod
        .as_str()
        .rfind(':')
        .ok_or_else(|| DfxError::MalformedFrontendUrl(frontend_url))?;
    frontend_url_mod.replace_range((port_offset + 1).., port.as_str());
    ping_and_wait(&frontend_url_mod)
}

// TODO(eftychis)/In progress: Rename to replica.
/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let temp_dir = env.get_temp_dir();

    let (frontend_url, address_and_port) = frontend_address(args, &config)?;
    let webserver_port_path = temp_dir.join("webserver-port");
    std::fs::write(&webserver_port_path, "")?;

    // don't write to file since this arg means we send_background()
    if !args.is_present("background") {
        std::fs::write(&webserver_port_path, address_and_port.port().to_string())?;
    }

    let temp_dir = env.get_temp_dir();

    let state_root = env.get_state_dir();

    let pid_file_path = temp_dir.join("pid");
    check_previous_process_running(&pid_file_path)?;

    // We are doing this here to make sure we can write to the temp
    // pid file.
    std::fs::write(&pid_file_path, "")?;

    if args.is_present("background") {
        send_background()?;
        return fg_ping_and_wait(webserver_port_path, frontend_url);
    }

    // As we know no start process is running in this project, we can
    // clean up the state if it is necessary.
    if args.is_present("clean") {
        clean_state(temp_dir, &state_root)?;
    }

    // We are doing this here to make sure we can write to the temp
    // pid file.
    std::fs::write(&pid_file_path, "")?;

    let system = actix::System::new("dfx-start");

    let shutdown_controller = ShutdownController::new(shutdown_controller::Config {
        logger: Some(env.get_logger().clone()),
    })
    .start();

    let replica_addr = start_replica(env, &state_root, shutdown_controller)?;

    let _webserver_coordinator =
        start_webserver_coordinator(env, args, config, address_and_port, replica_addr)?;

    // Update the pid file.
    if let Ok(pid) = sysinfo::get_current_pid() {
        let _ = std::fs::write(&pid_file_path, pid.to_string());
    }

    system.run()?;

    Ok(())
}

fn clean_state(temp_dir: &Path, state_root: &Path) -> DfxResult {
    // Clean the contents of the provided directory including the
    // directory itself. N.B. This does NOT follow symbolic links -- and I
    // hope we do not need to.
    if state_root.is_dir() {
        fs::remove_dir_all(state_root)
            .map_err(|e| DfxError::CleanState(e, PathBuf::from(state_root)))?;
    }
    let local_dir = temp_dir.join("local");
    if local_dir.is_dir() {
        fs::remove_dir_all(&local_dir).map_err(|e| DfxError::CleanState(e, local_dir))?;
    }
    Ok(())
}

fn start_replica(
    env: &dyn Environment,
    state_root: &Path,
    shutdown_controller: Addr<ShutdownController>,
) -> DfxResult<Addr<Replica>> {
    let replica_pathbuf = env.get_cache().get_binary_command_path("replica")?;
    let ic_starter_pathbuf = env.get_cache().get_binary_command_path("ic-starter")?;

    let temp_dir = env.get_temp_dir();
    let client_configuration_dir = temp_dir.join("client-configuration");
    fs::create_dir_all(&client_configuration_dir)?;
    let state_dir = temp_dir.join("state/replicated_state");
    fs::create_dir_all(&state_dir)?;
    let client_port_path = client_configuration_dir.join("client-1.port");

    // Touch the client port file. This ensures it is empty prior to
    // handing it over to the replica. If we read the file and it has
    // contents we shall assume it is due to our spawned client
    // process.
    std::fs::write(&client_port_path, "")?;

    let replica_config = ReplicaConfig::new(state_root).with_random_port(&client_port_path);
    Ok(actors::replica::Replica::new(actors::replica::Config {
        ic_starter_path: ic_starter_pathbuf,
        replica_config,
        replica_path: replica_pathbuf,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
    })
    .start())
}

fn start_webserver_coordinator(
    env: &dyn Environment,
    args: &ArgMatches<'_>,
    config: Arc<Config>,
    address_and_port: SocketAddr,
    replica_addr: Addr<Replica>,
) -> DfxResult<Addr<ReplicaWebserverCoordinator>> {
    let network_descriptor = get_network_descriptor(env, args)?;
    let bootstrap_dir = env.get_cache().get_binary_command_path("bootstrap")?;
    // By default we reach to no external IC nodes.
    let providers = Vec::new();
    let build_output_root = config
        .get_temp_path()
        .join(network_descriptor.name.clone())
        .join("canisters");

    let coord_config = actors::replica_webserver_coordinator::Config {
        logger: Some(env.get_logger().clone()),
        replica_addr,
        bind: address_and_port,
        serve_dir: bootstrap_dir,
        providers,
        build_output_root,
        network_descriptor,
    };
    Ok(ReplicaWebserverCoordinator::new(coord_config).start())
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
    let mut address_and_port = args
        .value_of("host")
        .and_then(|host| Option::from(host.parse()))
        .unwrap_or_else(|| {
            Ok(config
                .get_config()
                .get_local_bind_address("localhost:8000")
                .expect("could not get socket_addr"))
        })
        .map_err(|e| DfxError::InvalidArgument(format!("Invalid host: {}", e)))?;

    if !args.is_present("background") {
        // Since the user may have provided port "0", we need to grab a dynamically
        // allocated port and construct a resuable SocketAddr which the actix
        // HttpServer will bind to
        address_and_port =
            get_reusable_socket_addr(address_and_port.ip(), address_and_port.port())?;
    }
    let ip = if address_and_port.is_ipv6() {
        format!("[{}]", address_and_port.ip())
    } else {
        address_and_port.ip().to_string()
    };
    let frontend_url = format!("http://{}:{}", ip, address_and_port.port());
    Ok((frontend_url, address_and_port))
}

fn check_previous_process_running(dfx_pid_path: &Path) -> DfxResult<()> {
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
