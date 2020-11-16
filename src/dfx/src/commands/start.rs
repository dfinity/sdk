use crate::actors;
use crate::actors::replica::Replica;
use crate::actors::replica_webserver_coordinator::ReplicaWebserverCoordinator;
use crate::actors::shutdown_controller;
use crate::actors::shutdown_controller::ShutdownController;
use crate::config::dfinity::Config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::get_network_descriptor;
use crate::lib::replica_config::ReplicaConfig;
use crate::util::get_reusable_socket_addr;

use actix::{Actor, Addr};
use anyhow::{anyhow, bail, Context};
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use delay::{Delay, Waiter};
use ic_agent::Agent;
use std::fs;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{System, SystemExt};
use tokio::runtime::Runtime;

/// Starts the local replica and a web server for the current project.
#[derive(Clap)]
#[clap(name("start"))]
pub struct StartOpts {
    /// Specifies the host name and port number to bind the frontend to.
    #[clap(long)]
    host: Option<String>,

    /// Exits the dfx leaving the replica running. Will wait until the replica replies before exiting.
    #[clap(long)]
    background: bool,

    /// Cleans the state of the current project.
    #[clap(long)]
    clean: bool,
}

pub fn construct() -> App<'static> {
    StartOpts::into_app()
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
                .map_err(|_| DfxError::new(status.unwrap_err()))?;
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
            waiter.wait().map_err(|err| anyhow!("{:?}", err))?;
        }
        Ok::<String, DfxError>(contents.clone())
    })?;
    let mut frontend_url_mod = frontend_url.clone();
    let port_offset = frontend_url_mod
        .as_str()
        .rfind(':')
        .ok_or_else(|| anyhow!("Malformed frontend url: {}", frontend_url))?;
    frontend_url_mod.replace_range((port_offset + 1).., port.as_str());
    ping_and_wait(&frontend_url_mod)
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: StartOpts = StartOpts::from_arg_matches(args);
    let config = env.get_config_or_anyhow()?;

    let network_descriptor = get_network_descriptor(env, None)?;

    let temp_dir = env.get_temp_dir();
    let build_output_root = temp_dir.join(&network_descriptor.name).join("canisters");
    let pid_file_path = temp_dir.join("pid");
    let webserver_port_path = temp_dir.join("webserver-port");
    let state_root = env.get_state_dir();

    check_previous_process_running(&pid_file_path)?;

    // As we know no start process is running in this project, we can
    // clean up the state if it is necessary.
    if opts.clean {
        clean_state(temp_dir, &state_root)?;
    }

    std::fs::write(&pid_file_path, "")?; // make sure we can write to this file
    std::fs::write(&webserver_port_path, "")?;

    let background = opts.background;
    let (frontend_url, address_and_port) = frontend_address(opts.host, &config, background)?;

    if background {
        send_background()?;
        return fg_ping_and_wait(webserver_port_path, frontend_url);
    }

    write_pid(&pid_file_path);
    std::fs::write(&webserver_port_path, address_and_port.port().to_string())?;

    let system = actix::System::new("dfx-start");

    let shutdown_controller = start_shutdown_controller(env)?;

    let replica = start_replica(env, &state_root, shutdown_controller.clone())?;

    let _webserver_coordinator = start_webserver_coordinator(
        env,
        network_descriptor,
        address_and_port,
        build_output_root,
        replica,
        shutdown_controller,
    )?;

    system.run()?;

    Ok(())
}

fn clean_state(temp_dir: &Path, state_root: &Path) -> DfxResult {
    // Clean the contents of the provided directory including the
    // directory itself. N.B. This does NOT follow symbolic links -- and I
    // hope we do not need to.
    if state_root.is_dir() {
        fs::remove_dir_all(state_root).context(format!(
            "Cannot remove directroy at '{}'.",
            state_root.display()
        ))?;
    }
    let local_dir = temp_dir.join("local");
    if local_dir.is_dir() {
        fs::remove_dir_all(&local_dir).context(format!(
            "Cannot remove directroy at '{}'.",
            local_dir.display()
        ))?;
    }
    Ok(())
}

fn start_shutdown_controller(env: &dyn Environment) -> DfxResult<Addr<ShutdownController>> {
    let actor_config = shutdown_controller::Config {
        logger: Some(env.get_logger().clone()),
    };
    Ok(ShutdownController::new(actor_config).start())
}

fn start_replica(
    env: &dyn Environment,
    state_root: &Path,
    shutdown_controller: Addr<ShutdownController>,
) -> DfxResult<Addr<Replica>> {
    let replica_path = env.get_cache().get_binary_command_path("replica")?;
    let ic_starter_path = env.get_cache().get_binary_command_path("ic-starter")?;

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
    let actor_config = actors::replica::Config {
        ic_starter_path,
        replica_config,
        replica_path,
        shutdown_controller,
        logger: Some(env.get_logger().clone()),
    };
    Ok(actors::replica::Replica::new(actor_config).start())
}

fn start_webserver_coordinator(
    env: &dyn Environment,
    network_descriptor: NetworkDescriptor,
    bind: SocketAddr,
    build_output_root: PathBuf,
    replica_addr: Addr<Replica>,
    shutdown_controller: Addr<ShutdownController>,
) -> DfxResult<Addr<ReplicaWebserverCoordinator>> {
    let serve_dir = env.get_cache().get_binary_command_path("bootstrap")?;
    // By default we reach to no external IC nodes.
    let providers = Vec::new();

    let actor_config = actors::replica_webserver_coordinator::Config {
        logger: Some(env.get_logger().clone()),
        replica_addr,
        shutdown_controller,
        bind,
        serve_dir,
        providers,
        build_output_root,
        network_descriptor,
    };
    Ok(ReplicaWebserverCoordinator::new(actor_config).start())
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

fn frontend_address(
    host: Option<String>,
    config: &Config,
    background: bool,
) -> DfxResult<(String, SocketAddr)> {
    let mut address_and_port = host
        .and_then(|host| Option::from(host.parse()))
        .unwrap_or_else(|| {
            Ok(config
                .get_config()
                .get_local_bind_address("localhost:8000")
                .expect("could not get socket_addr"))
        })
        .map_err(|e| anyhow!("Invalid argument: Invalid host: {}", e))?;

    if !background {
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
                    bail!("dfx is already running.");
                }
            }
        }
    }
    Ok(())
}

fn write_pid(pid_file_path: &Path) {
    if let Ok(pid) = sysinfo::get_current_pid() {
        let _ = std::fs::write(&pid_file_path, pid.to_string());
    }
}
