use crate::actors::icx_proxy::signals::PortReadySubscribe;
use crate::actors::{
    start_btc_adapter_actor, start_canister_http_adapter_actor, start_emulator_actor,
    start_icx_proxy_actor, start_replica_actor, start_shutdown_controller,
};
use crate::config::dfinity::ConfigInterface;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::replica_config::ReplicaConfig;
use crate::lib::{bitcoin, canister_http};
use crate::util::get_reusable_socket_addr;

use crate::actors::icx_proxy::IcxProxyConfig;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use crate::lib::provider::get_network_descriptor;
use crate::lib::webserver::run_webserver;
use actix::Recipient;
use anyhow::{anyhow, bail, Context, Error};
use clap::Parser;
use fn_error_context::context;
use garcon::{Delay, Waiter};
use ic_agent::Agent;
use std::fs;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, System, SystemExt};
use tokio::runtime::Runtime;

/// Starts the local replica and a web server for the current project.
#[derive(Parser)]
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

    /// Address of bitcoind node.  Implies --enable-bitcoin.
    #[clap(long, conflicts_with("emulator"), multiple_occurrences(true))]
    bitcoin_node: Vec<SocketAddr>,

    /// enable bitcoin integration
    #[clap(long, conflicts_with("emulator"))]
    enable_bitcoin: bool,

    /// enable canister http requests
    #[clap(long, conflicts_with("emulator"))]
    enable_canister_http: bool,
}

fn ping_and_wait(frontend_url: &str) -> DfxResult {
    let runtime = Runtime::new().expect("Unable to create a runtime");

    let agent = Agent::builder()
        .with_transport(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(frontend_url)
                .with_context(|| {
                    format!(
                        "Failed to create replica transport from frontend url {}.",
                        frontend_url
                    )
                })?,
        )
        .build()
        .with_context(|| format!("Failed to build agent with frontend url {}.", frontend_url))?;

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
    let port = runtime
        .block_on(async {
            waiter.start();
            let mut contents = String::new();
            loop {
                let tokio_file = tokio::fs::File::open(&webserver_port_path)
                    .await
                    .with_context(|| {
                        format!("Failed to open {}.", webserver_port_path.to_string_lossy())
                    })?;
                let mut std_file = tokio_file.into_std().await;
                std_file.read_to_string(&mut contents).with_context(|| {
                    format!("Failed to read {}.", webserver_port_path.to_string_lossy())
                })?;
                if !contents.is_empty() {
                    break;
                }
                waiter.wait().map_err(|err| anyhow!("{:?}", err))?;
            }
            Ok::<String, DfxError>(contents.clone())
        })
        .context("Failed to get port.")?;
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
pub fn exec(
    env: &dyn Environment,
    StartOpts {
        host,
        background,
        emulator,
        clean,
        bitcoin_node,
        enable_bitcoin,
        enable_canister_http,
    }: StartOpts,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let network_descriptor = get_network_descriptor(env.get_config(), None)?;
    let local_server_descriptor = network_descriptor.local_server_descriptor()?;
    let temp_dir = env.get_temp_dir();
    let build_output_root = temp_dir.join(&network_descriptor.name).join("canisters");
    let pid_file_path = temp_dir.join("pid");
    let state_root = env.get_state_dir();

    check_previous_process_running(&pid_file_path)?;

    let btc_adapter_socket_holder_path = temp_dir.join("ic-btc-adapter-socket-path");
    let canister_http_adapter_socket_holder_path = temp_dir.join("ic-canister-http-socket-path");

    // As we know no start process is running in this project, we can
    // clean up the state if it is necessary.
    if clean {
        clean_state(temp_dir, &state_root)?;
    }

    let pid_file_path = empty_writable_path(pid_file_path)?;
    let btc_adapter_pid_file_path = empty_writable_path(temp_dir.join("ic-btc-adapter-pid"))?;
    let btc_adapter_config_path = empty_writable_path(temp_dir.join("ic-btc-adapter-config.json"))?;
    let canister_http_adapter_pid_file_path =
        empty_writable_path(temp_dir.join("ic-canister-http-adapter-pid"))?;
    let canister_http_adapter_config_path =
        empty_writable_path(temp_dir.join("ic-canister-http-config.json"))?;
    let icx_proxy_pid_file_path = empty_writable_path(temp_dir.join("icx-proxy-pid"))?;
    let webserver_port_path = empty_writable_path(temp_dir.join("webserver-port"))?;

    let (frontend_url, address_and_port) =
        frontend_address(host, local_server_descriptor, background)?;

    if background {
        send_background()?;
        return fg_ping_and_wait(webserver_port_path, frontend_url);
    }

    write_pid(&pid_file_path);
    std::fs::write(&webserver_port_path, address_and_port.port().to_string()).with_context(
        || {
            format!(
                "Failed to write webserver port file {}.",
                webserver_port_path.to_string_lossy()
            )
        },
    )?;

    let btc_adapter_config = configure_btc_adapter_if_enabled(
        config.get_config(),
        &btc_adapter_config_path,
        &btc_adapter_socket_holder_path,
        enable_bitcoin,
        bitcoin_node,
    )?;
    let btc_adapter_socket_path = btc_adapter_config
        .as_ref()
        .and_then(|cfg| cfg.get_socket_path());

    let canister_http_adapter_config = configure_canister_http_adapter_if_enabled(
        config.get_config(),
        &canister_http_adapter_config_path,
        &canister_http_adapter_socket_holder_path,
        enable_canister_http,
    )?;
    let canister_http_socket_path = canister_http_adapter_config
        .as_ref()
        .and_then(|cfg| cfg.get_socket_path());

    let system = actix::System::new();
    let _proxy = system.block_on(async move {
        let shutdown_controller = start_shutdown_controller(env)?;

        let port_ready_subscribe: Recipient<PortReadySubscribe> = if emulator {
            let emulator = start_emulator_actor(env, shutdown_controller.clone())?;
            emulator.recipient()
        } else {
            let (btc_adapter_ready_subscribe, btc_adapter_socket_path) =
                if let Some(ref btc_adapter_config) = btc_adapter_config {
                    let socket_path = btc_adapter_config.get_socket_path();
                    let ready_subscribe = start_btc_adapter_actor(
                        env,
                        btc_adapter_config_path,
                        socket_path.clone(),
                        shutdown_controller.clone(),
                        btc_adapter_pid_file_path,
                    )?
                    .recipient();
                    (Some(ready_subscribe), socket_path)
                } else {
                    (None, None)
                };
            let (canister_http_adapter_ready_subscribe, canister_http_socket_path) =
                if let Some(ref canister_http_adapter_config) = canister_http_adapter_config {
                    let socket_path = canister_http_adapter_config.get_socket_path();
                    let ready_subscribe = start_canister_http_adapter_actor(
                        env,
                        canister_http_adapter_config_path,
                        socket_path.clone(),
                        shutdown_controller.clone(),
                        canister_http_adapter_pid_file_path,
                    )?
                    .recipient();
                    (Some(ready_subscribe), socket_path)
                } else {
                    (None, None)
                };

            let replica_port_path = env
                .get_temp_dir()
                .join("replica-configuration")
                .join("replica-1.port");

            let subnet_type = config
                .get_config()
                .get_defaults()
                .get_replica()
                .subnet_type
                .unwrap_or_default();
            let mut replica_config = ReplicaConfig::new(&env.get_state_dir(), subnet_type)
                .with_random_port(&replica_port_path);
            if btc_adapter_config.is_some() {
                replica_config = replica_config.with_btc_adapter_enabled();
                if let Some(btc_adapter_socket) = btc_adapter_socket_path {
                    replica_config = replica_config.with_btc_adapter_socket(btc_adapter_socket);
                }
            }
            if canister_http_adapter_config.is_some() {
                replica_config = replica_config.with_canister_http_adapter_enabled();
                if let Some(socket_path) = canister_http_socket_path {
                    replica_config = replica_config.with_canister_http_adapter_socket(socket_path);
                }
            }

            let replica = start_replica_actor(
                env,
                replica_config,
                shutdown_controller.clone(),
                btc_adapter_ready_subscribe,
                canister_http_adapter_ready_subscribe,
            )?;
            replica.recipient()
        };

        let webserver_bind = get_reusable_socket_addr(address_and_port.ip(), 0)?;
        let icx_proxy_config = IcxProxyConfig {
            bind: address_and_port,
            proxy_port: webserver_bind.port(),
            providers: vec![],
            fetch_root_key: !network_descriptor.is_ic,
        };

        run_webserver(
            env.get_logger().clone(),
            build_output_root,
            network_descriptor,
            config.get_project_root().to_path_buf(),
            env.get_temp_dir().to_path_buf(),
            webserver_bind,
        )?;

        let proxy = start_icx_proxy_actor(
            env,
            icx_proxy_config,
            Some(port_ready_subscribe),
            shutdown_controller,
            icx_proxy_pid_file_path,
        )?;
        Ok::<_, Error>(proxy)
    })?;
    system.run()?;

    if let Some(btc_adapter_socket_path) = btc_adapter_socket_path {
        let _ = std::fs::remove_file(&btc_adapter_socket_path);
    }
    if let Some(canister_http_socket_path) = canister_http_socket_path {
        let _ = std::fs::remove_file(&canister_http_socket_path);
    }

    Ok(())
}

#[context("Failed to clean existing replica state.")]
fn clean_state(temp_dir: &Path, state_root: &Path) -> DfxResult {
    // Clean the contents of the provided directory including the
    // directory itself. N.B. This does NOT follow symbolic links -- and I
    // hope we do not need to.
    if state_root.is_dir() {
        fs::remove_dir_all(state_root)
            .with_context(|| format!("Cannot remove directory at '{}'.", state_root.display()))?;
    }
    let local_dir = temp_dir.join("local");
    if local_dir.is_dir() {
        fs::remove_dir_all(&local_dir)
            .with_context(|| format!("Cannot remove directory at '{}'.", local_dir.display()))?;
    }
    Ok(())
}

#[context("Failed to spawn background dfx.")]
fn send_background() -> DfxResult<()> {
    // Background strategy is different; we spawn `dfx` with the same arguments
    // (minus --background), ping and exit.
    let exe = std::env::current_exe().context("Failed to get current executable.")?;
    let mut cmd = Command::new(exe);
    // Skip 1 because arg0 is this executable's path.
    cmd.args(std::env::args().skip(1).filter(|a| !a.eq("--background")));

    cmd.spawn().context("Failed to spawn child process.")?;
    Ok(())
}

#[context("Failed to get frontend address.")]
fn frontend_address(
    host: Option<String>,
    local_server_descriptor: &LocalServerDescriptor,
    background: bool,
) -> DfxResult<(String, SocketAddr)> {
    let address_and_port = host
        .and_then(|host| Option::from(host.parse()))
        .transpose()
        .map_err(|e| anyhow!("Invalid argument: Invalid host: {}", e))?;

    let mut address_and_port = address_and_port.unwrap_or(local_server_descriptor.bind_address);

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
            if let Ok(pid) = s.parse::<Pid>() {
                // If we find the pid in the file, we tell the user and don't start!
                let mut system = System::new();
                system.refresh_processes();
                if let Some(_process) = system.process(pid) {
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

#[context("Failed to configure btc adapter.")]
pub fn configure_btc_adapter_if_enabled(
    config: &ConfigInterface,
    config_path: &Path,
    uds_holder_path: &Path,
    enable_bitcoin: bool,
    nodes: Vec<SocketAddr>,
) -> DfxResult<Option<bitcoin::adapter::Config>> {
    let enable = enable_bitcoin || !nodes.is_empty() || config.get_defaults().get_bitcoin().enabled;

    if !enable {
        return Ok(None);
    };

    let nodes = match (nodes, &config.get_defaults().get_bitcoin().nodes) {
        (cli_nodes, _) if !cli_nodes.is_empty() => cli_nodes,
        (_, Some(default_nodes)) if !default_nodes.is_empty() => default_nodes.clone(),
        (_, _) => bitcoin::adapter::config::default_nodes(),
    };

    let config = write_btc_adapter_config(uds_holder_path, config_path, nodes)?;
    Ok(Some(config))
}

#[context("Failed to create persistent socket path for {} at {}.", prefix, uds_holder_path.to_string_lossy())]
fn create_new_persistent_socket_path(uds_holder_path: &Path, prefix: &str) -> DfxResult<PathBuf> {
    let pid = sysinfo::get_current_pid()
        .map_err(|s| anyhow!("Unable to obtain pid of current process: {}", s))?;
    let timestamp_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Unix domain socket names can only be so long.
    // An attempt to use a path under .dfx/ resulted in this error:
    //    path must be shorter than libc::sockaddr_un.sun_path
    let uds_path = format!("/tmp/{}.{}.{}", prefix, pid, timestamp_seconds);
    std::fs::write(uds_holder_path, &uds_path).with_context(|| {
        format!(
            "unable to write unix domain socket path to {}",
            uds_holder_path.to_string_lossy()
        )
    })?;
    Ok(PathBuf::from(uds_path))
}

#[context("Failed to get persistent socket path for {} at {}.", prefix, uds_holder_path.to_string_lossy())]
fn get_persistent_socket_path(uds_holder_path: &Path, prefix: &str) -> DfxResult<PathBuf> {
    if let Ok(uds_path) = std::fs::read_to_string(uds_holder_path) {
        Ok(PathBuf::from(uds_path.trim()))
    } else {
        create_new_persistent_socket_path(uds_holder_path, prefix)
    }
}

#[context("Failed to write btc adapter config to {}.", config_path.to_string_lossy())]
fn write_btc_adapter_config(
    uds_holder_path: &Path,
    config_path: &Path,
    nodes: Vec<SocketAddr>,
) -> DfxResult<bitcoin::adapter::Config> {
    let socket_path = get_persistent_socket_path(uds_holder_path, "ic-btc-adapter-socket")?;

    let adapter_config = bitcoin::adapter::Config::new(nodes, socket_path);

    let contents = serde_json::to_string_pretty(&adapter_config)
        .context("Unable to serialize btc adapter configuration to json")?;
    std::fs::write(config_path, &contents).with_context(|| {
        format!(
            "Unable to write btc adapter configuration to {}",
            config_path.to_string_lossy()
        )
    })?;

    Ok(adapter_config)
}

pub fn empty_writable_path(path: PathBuf) -> DfxResult<PathBuf> {
    std::fs::write(&path, "")
        .with_context(|| format!("Unable to write to {}", path.to_string_lossy()))?;
    Ok(path)
}

#[context("Failed to configure canister http adapter.")]
pub fn configure_canister_http_adapter_if_enabled(
    config: &ConfigInterface,
    config_path: &Path,
    uds_holder_path: &Path,
    enable_canister_http: bool,
) -> DfxResult<Option<canister_http::adapter::Config>> {
    let enable = enable_canister_http || config.get_defaults().get_canister_http().enabled;

    if !enable {
        return Ok(None);
    };

    let socket_path =
        get_persistent_socket_path(uds_holder_path, "ic-canister-http-adapter-socket")?;

    let adapter_config = canister_http::adapter::Config::new(socket_path);

    let contents = serde_json::to_string_pretty(&adapter_config)
        .context("Unable to serialize canister http adapter configuration to json")?;
    std::fs::write(config_path, &contents)
        .with_context(|| format!("Unable to write {}", config_path.to_string_lossy()))?;

    Ok(Some(adapter_config))
}
