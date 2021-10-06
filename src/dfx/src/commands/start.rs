use crate::actors;
use crate::actors::icx_proxy::signals::PortReadySubscribe;
use crate::actors::{
    start_emulator_actor, start_icx_proxy_actor, start_replica_actor, start_shutdown_controller,
};
use crate::config::dfinity::Config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::replica_config::ReplicaConfig;
use crate::util::get_reusable_socket_addr;

use crate::actors::icx_proxy::IcxProxyConfig;
use crate::actors::proxy_webserver_coordinator::ProxyWebserverCoordinator;
use crate::actors::shutdown_controller::ShutdownController;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::get_network_descriptor;
use actix::{Actor, Addr, Recipient};
use anyhow::{anyhow, bail, Context};
use clap::Clap;
use garcon::{Delay, Waiter};
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

    /// Runs a dedicated emulator instead of the replica
    #[clap(long)]
    emulator: bool,

    /// Removes the artificial delay in the local replica added to simulate the networked IC environment.
    #[clap(long)]
    no_artificial_delay: bool,
}

fn ping_and_wait(frontend_url: &str) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");

    let agent = Agent::builder()
        .with_transport(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(frontend_url)?,
        )
        .build()?;

    // wait for frontend to come up
    let mut waiter = Delay::builder()
        .timeout(std::time::Duration::from_secs(60))
        .throttle(std::time::Duration::from_secs(1))
        .build();

    runtime.block_on(async {
        waiter.start();
        loop {
            let status = agent.status().await;
            if let Ok(status) = &status {
                let healthy = match &status.replica_health_status {
                    Some(status) if status == "healthy" => true,
                    None => true, // emulator doesn't report replica_health_status
                    _ => false,
                };
                if healthy {
                    break;
                }
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
    let runtime = Runtime::new().expect("Unable to create a runtime");
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
pub fn exec(env: &dyn Environment, opts: StartOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let network_descriptor = get_network_descriptor(env, None)?;
    let temp_dir = env.get_temp_dir();
    let build_output_root = temp_dir.join(&network_descriptor.name).join("canisters");
    let pid_file_path = temp_dir.join("pid");
    let icx_proxy_pid_file_path = temp_dir.join("icx-proxy-pid");
    let webserver_port_path = temp_dir.join("webserver-port");
    let state_root = env.get_state_dir();

    check_previous_process_running(&pid_file_path)?;

    // As we know no start process is running in this project, we can
    // clean up the state if it is necessary.
    if opts.clean {
        clean_state(temp_dir, &state_root)?;
    }

    std::fs::write(&pid_file_path, "")?; // make sure we can write to this file
    std::fs::write(&icx_proxy_pid_file_path, "")?;
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

    let port_ready_subscribe: Recipient<PortReadySubscribe> = if opts.emulator {
        let emulator = start_emulator_actor(env, shutdown_controller.clone())?;
        emulator.recipient()
    } else {
        let replica_port_path = env
            .get_temp_dir()
            .join("replica-configuration")
            .join("replica-1.port");

        let replica_config = ReplicaConfig::new(&env.get_state_dir(), opts.no_artificial_delay)
            .with_random_port(&replica_port_path);
        let replica = start_replica_actor(env, replica_config, shutdown_controller.clone())?;
        replica.recipient()
    };

    let webserver_bind = get_reusable_socket_addr(address_and_port.ip(), 0)?;

    let _webserver_coordinator = start_webserver_coordinator(
        env,
        network_descriptor,
        webserver_bind,
        build_output_root,
        shutdown_controller.clone(),
    )?;

    let icx_proxy_config = IcxProxyConfig {
        bind: address_and_port,
        proxy_port: webserver_bind.port(),
        providers: vec![],
    };
    let _proxy = start_icx_proxy_actor(
        env,
        icx_proxy_config,
        Some(port_ready_subscribe),
        shutdown_controller,
        icx_proxy_pid_file_path,
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
            "Cannot remove directory at '{}'.",
            state_root.display()
        ))?;
    }
    let local_dir = temp_dir.join("local");
    if local_dir.is_dir() {
        fs::remove_dir_all(&local_dir).context(format!(
            "Cannot remove directory at '{}'.",
            local_dir.display()
        ))?;
    }
    Ok(())
}

pub fn start_webserver_coordinator(
    env: &dyn Environment,
    network_descriptor: NetworkDescriptor,
    bind: SocketAddr,
    build_output_root: PathBuf,
    shutdown_controller: Addr<ShutdownController>,
) -> DfxResult<Addr<ProxyWebserverCoordinator>> {
    let actor_config = actors::proxy_webserver_coordinator::Config {
        logger: Some(env.get_logger().clone()),
        shutdown_controller,
        bind,
        build_output_root,
        network_descriptor,
    };
    Ok(ProxyWebserverCoordinator::new(actor_config).start())
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
