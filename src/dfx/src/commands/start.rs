use crate::actors::icx_proxy::signals::PortReadySubscribe;
use crate::actors::{
    start_btc_adapter_actor, start_emulator_actor, start_icx_proxy_actor, start_replica_actor,
    start_shutdown_controller,
};
use crate::config::dfinity::Config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::replica_config::ReplicaConfig;
use crate::util::get_reusable_socket_addr;

use crate::actors::icx_proxy::IcxProxyConfig;
use crate::lib::provider::get_network_descriptor;
use crate::lib::webserver::run_webserver;
use actix::Recipient;
use anyhow::{anyhow, bail, Context, Error};
use clap::Parser;
use garcon::{Delay, Waiter};
use ic_agent::Agent;
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
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

    /// The path of the btc adapter configuration file.  Implies --enable-bitcoin.
    #[clap(long, conflicts_with("emulator"))]
    btc_adapter_config: Option<PathBuf>,

    /// enable the bitcoin adapter
    #[clap(long)]
    enable_bitcoin: bool,
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
        btc_adapter_config,
        enable_bitcoin,
    }: StartOpts,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let network_descriptor =
        get_network_descriptor(env, None).context("Failed to get network descriptor.")?;
    let temp_dir = env.get_temp_dir();
    let build_output_root = temp_dir.join(&network_descriptor.name).join("canisters");
    let pid_file_path = temp_dir.join("pid");
    let btc_adapter_pid_file_path = temp_dir.join("ic-btc-adapter-pid");
    let icx_proxy_pid_file_path = temp_dir.join("icx-proxy-pid");
    let webserver_port_path = temp_dir.join("webserver-port");
    let state_root = env.get_state_dir();

    check_previous_process_running(&pid_file_path)?;

    // As we know no start process is running in this project, we can
    // clean up the state if it is necessary.
    if clean {
        clean_state(temp_dir, &state_root).context("Failed to clean up existing state.")?;
    }

    std::fs::write(&pid_file_path, "").with_context(|| {
        format!(
            "Failed to create/clear pid file {}.",
            pid_file_path.to_string_lossy()
        )
    })?;
    std::fs::write(&btc_adapter_pid_file_path, "").with_context(|| {
        format!(
            "Failed to create/clear BTC adapter pid file {}.",
            btc_adapter_pid_file_path.to_string_lossy()
        )
    })?;
    std::fs::write(&icx_proxy_pid_file_path, "").with_context(|| {
        format!(
            "Failed to create/clear icx proxy pid file {}.",
            icx_proxy_pid_file_path.to_string_lossy()
        )
    })?;
    std::fs::write(&webserver_port_path, "").with_context(|| {
        format!(
            "Failed to create/clear webserver port file {}.",
            webserver_port_path.to_string_lossy()
        )
    })?;

    let (frontend_url, address_and_port) =
        frontend_address(host, &config, background).context("Failed to get frontend address.")?;

    if background {
        send_background().context("Failed to spawn background dfx.")?;
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

    let btc_adapter_config = get_btc_adapter_config(&config, enable_bitcoin, btc_adapter_config)
        .context("Failed to get BTC adapter config.")?;

    let system = actix::System::new();
    let _proxy = system
        .block_on(async move {
            let shutdown_controller =
                start_shutdown_controller(env).context("Failed to start shutdown controller.")?;

            let port_ready_subscribe: Recipient<PortReadySubscribe> = if emulator {
                let emulator = start_emulator_actor(env, shutdown_controller.clone())
                    .context("Failed to start emulator actor.")?;
                emulator.recipient()
            } else {
                let (btc_adapter_ready_subscribe, btc_adapter_socket_path) =
                    if let Some(btc_adapter_config) = btc_adapter_config {
                        let socket_path = get_btc_adapter_socket_path(&btc_adapter_config)
                            .with_context(|| {
                                format!(
                                    "Failed to get BTC adapter socket path from config at {}.",
                                    btc_adapter_config.to_string_lossy()
                                )
                            })?;
                        let ready_subscribe = start_btc_adapter_actor(
                            env,
                            btc_adapter_config,
                            socket_path.clone(),
                            shutdown_controller.clone(),
                            btc_adapter_pid_file_path,
                        )
                        .context("Failed to start BTC adapter actor.")?
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
                if let Some(btc_adapter_socket) = btc_adapter_socket_path {
                    replica_config = replica_config.with_btc_adapter_socket(btc_adapter_socket);
                }

                let replica = start_replica_actor(
                    env,
                    replica_config,
                    shutdown_controller.clone(),
                    btc_adapter_ready_subscribe,
                )
                .context("Failed to start replica actor.")?;
                replica.recipient()
            };

            let webserver_bind = get_reusable_socket_addr(address_and_port.ip(), 0)
                .context("Failed to find reusable socket address.")?;
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
                webserver_bind,
            )
            .context("Failed to start webserver.")?;

            let proxy = start_icx_proxy_actor(
                env,
                icx_proxy_config,
                Some(port_ready_subscribe),
                shutdown_controller,
                icx_proxy_pid_file_path,
            )
            .context("Failed to start icx proxy.")?;
            Ok::<_, Error>(proxy)
        })
        .context("Failed to setup system.")?;
    system.run().context("Failed to run system.")?;
    Ok(())
}

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
        address_and_port = get_reusable_socket_addr(address_and_port.ip(), address_and_port.port())
            .with_context(|| format!("Failed to get frontend address {}", address_and_port))?;
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

pub fn get_btc_adapter_socket_path(btc_config_path: &Path) -> DfxResult<Option<PathBuf>> {
    let content = std::fs::read(&btc_config_path)
        .with_context(|| format!("Unable to read {}", btc_config_path.to_string_lossy()))?;
    let json: Value = serde_json::from_slice(&content).with_context(|| {
        format!(
            "Unable to decode {} as json",
            btc_config_path.to_string_lossy()
        )
    })?;
    Ok(json
        .pointer("/incoming_source/Path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from))
}

pub fn get_btc_adapter_config(
    config: &Arc<Config>,
    enable_bitcoin: bool,
    btc_adapter_config: Option<PathBuf>,
) -> DfxResult<Option<PathBuf>> {
    let btc_adapter_config: Option<PathBuf> = {
        let enable = enable_bitcoin
            || btc_adapter_config.is_some()
            || config.get_config().get_defaults().get_bitcoin().enabled;
        let config = btc_adapter_config.or_else(|| {
            config
                .get_config()
                .get_defaults()
                .bitcoin
                .as_ref()
                .and_then(|x| x.btc_adapter_config.clone())
        });

        match (enable, config) {
            (true, Some(path)) => Some(path),
            (true, None) => {
                bail!("Bitcoin integration was enabled without either --btc-adapter-config or .defaults.bitcoin.btc_adapter_config in dfx.json")
            }
            (false, _) => None,
        }
    };
    Ok(btc_adapter_config)
}
