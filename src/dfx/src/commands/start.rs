use crate::actors::icx_proxy::signals::PortReadySubscribe;
use crate::actors::icx_proxy::IcxProxyConfig;
use crate::actors::{
    start_btc_adapter_actor, start_canister_http_adapter_actor, start_icx_proxy_actor,
    start_replica_actor, start_shutdown_controller,
};
use crate::config::dfx_version_str;
use crate::error_invalid_argument;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::info::replica_rev;
use crate::lib::integrations::status::wait_for_integrations_initialized;
use crate::lib::network::id::write_network_id;
use crate::lib::replica::status::ping_and_wait;
use crate::lib::replica_config::ReplicaConfig;
use crate::util::get_reusable_socket_addr;
use actix::Recipient;
use anyhow::{anyhow, bail, Context, Error};
use candid::Deserialize;
use clap::{ArgAction, Parser};
use dfx_core::config::model::local_server_descriptor::LocalServerDescriptor;
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::config::model::{bitcoin_adapter, canister_http_adapter};
use dfx_core::json::{load_json_file, save_json_file};
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};
use fn_error_context::context;
use os_str_bytes::{OsStrBytes, OsStringBytes};
use serde::Serialize;
use slog::{info, warn, Logger};
use std::borrow::Cow;
use std::fs;
use std::fs::create_dir_all;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, System, SystemExt};
use tokio::runtime::Runtime;

/// Starts the local replica and a web server for the current project.
#[derive(Parser)]
pub struct StartOpts {
    /// Specifies the host name and port number to bind the frontend to.
    #[arg(long)]
    host: Option<String>,

    /// Exits the dfx leaving the replica running. Will wait until the replica replies before exiting.
    #[arg(long)]
    background: bool,

    /// Cleans the state of the current project.
    #[arg(long)]
    clean: bool,

    /// Address of bitcoind node.  Implies --enable-bitcoin.
    #[arg(long, action = ArgAction::Append)]
    bitcoin_node: Vec<SocketAddr>,

    /// enable bitcoin integration
    #[arg(long)]
    enable_bitcoin: bool,

    /// enable canister http requests
    #[arg(long)]
    enable_canister_http: bool,

    /// The delay (in milliseconds) an update call should take. Lower values may be expedient in CI.
    #[arg(long, default_value_t = 600)]
    artificial_delay: u32,

    /// Start even if the network config was modified.
    #[arg(long)]
    force: bool,

    /// Use old metering.
    #[arg(long)]
    use_old_metering: bool,

    /// A list of domains that can be served. These are used for canister resolution [default: localhost]
    #[arg(long)]
    domain: Vec<String>,
}

// The frontend webserver is brought up by the bg process; thus, the fg process
// needs to wait and verify it's up before exiting.
// Because the user may have specified to start on port 0, here we wait for
// webserver_port_path to get written to and modify the frontend_url so we
// ping the correct address.
async fn fg_ping_and_wait(
    webserver_port_path: &Path,
    frontend_url: &str,
    logger: &Logger,
    local_server_descriptor: &LocalServerDescriptor,
) -> DfxResult {
    let port = wait_for_port(webserver_port_path).await?;

    let mut frontend_url_mod = frontend_url.to_string();
    let port_offset = frontend_url_mod
        .as_str()
        .rfind(':')
        .ok_or_else(|| anyhow!("Malformed frontend url: {}", frontend_url))?;
    frontend_url_mod.replace_range((port_offset + 1).., port.as_str());
    ping_and_wait(&frontend_url_mod).await?;

    wait_for_integrations_initialized(&frontend_url_mod, logger, local_server_descriptor).await
}

async fn wait_for_port(webserver_port_path: &Path) -> DfxResult<String> {
    let mut retries = 0;
    loop {
        let tokio_file = tokio::fs::File::open(&webserver_port_path)
            .await
            .with_context(|| {
                format!("Failed to open {}.", webserver_port_path.to_string_lossy())
            })?;
        let mut std_file = tokio_file.into_std().await;
        let mut contents = String::new();
        std_file.read_to_string(&mut contents).with_context(|| {
            format!("Failed to read {}.", webserver_port_path.to_string_lossy())
        })?;
        if !contents.is_empty() {
            break Ok(contents);
        }
        if retries >= 30 {
            bail!("Timed out waiting for replica to become healthy");
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        retries += 1;
    }
}

/// Start the Internet Computer locally. Spawns a proxy to forward and
/// manage browser requests. Responsible for running the network (one
/// replica at the moment) and the proxy.
pub fn exec(
    env: &dyn Environment,
    StartOpts {
        host,
        background,
        clean,
        force,
        bitcoin_node,
        enable_bitcoin,
        enable_canister_http,
        artificial_delay,
        use_old_metering,
        domain,
    }: StartOpts,
) -> DfxResult {
    if !background {
        info!(
            env.get_logger(),
            "Running dfx start for version {}",
            dfx_version_str()
        );
    }
    let project_config = env.get_config()?;

    let network_descriptor_logger = if background {
        None // so we don't print it out twice
    } else {
        Some(env.get_logger().clone())
    };
    let network_descriptor = create_network_descriptor(
        project_config,
        env.get_networks_config(),
        None,
        network_descriptor_logger,
        LocalBindDetermination::AsConfigured,
    )?;

    let network_descriptor = apply_command_line_parameters(
        env.get_logger(),
        network_descriptor,
        host,
        None,
        enable_bitcoin,
        bitcoin_node,
        enable_canister_http,
        domain,
    )?;

    let local_server_descriptor = network_descriptor.local_server_descriptor()?;
    let pid_file_path = local_server_descriptor.dfx_pid_path();

    check_previous_process_running(local_server_descriptor)?;

    // As we know no start process is running in this project, we can
    // clean up the state if it is necessary.
    if clean {
        clean_state(local_server_descriptor, env.get_project_temp_dir()?)?;
    }

    let (frontend_url, address_and_port) = frontend_address(local_server_descriptor, background)?;

    let network_temp_dir = local_server_descriptor.data_directory.clone();
    create_dir_all(&network_temp_dir).with_context(|| {
        format!(
            "Failed to create network temp directory {}.",
            network_temp_dir.to_string_lossy()
        )
    })?;

    if !local_server_descriptor.network_id_path().exists() {
        write_network_id(local_server_descriptor)?;
    }

    let state_root = local_server_descriptor.state_dir();

    let btc_adapter_socket_holder_path = local_server_descriptor.btc_adapter_socket_holder_path();
    let canister_http_adapter_socket_holder_path =
        local_server_descriptor.canister_http_adapter_socket_holder_path();

    let pid_file_path = empty_writable_path(pid_file_path)?;
    let btc_adapter_pid_file_path =
        empty_writable_path(local_server_descriptor.btc_adapter_pid_path())?;
    let btc_adapter_config_path =
        empty_writable_path(local_server_descriptor.btc_adapter_config_path())?;
    let canister_http_adapter_pid_file_path =
        empty_writable_path(local_server_descriptor.canister_http_adapter_pid_path())?;
    let canister_http_adapter_config_path =
        empty_writable_path(local_server_descriptor.canister_http_adapter_config_path())?;
    let icx_proxy_pid_file_path =
        empty_writable_path(local_server_descriptor.icx_proxy_pid_path())?;
    let webserver_port_path = empty_writable_path(local_server_descriptor.webserver_port_path())?;

    let previous_config_path = local_server_descriptor.effective_config_path();

    // dfx info replica-port will read these port files to find out which port to use,
    // so we need to make sure only one has a valid port in it.
    let replica_config_dir = local_server_descriptor.replica_configuration_dir();
    fs::create_dir_all(&replica_config_dir).with_context(|| {
        format!(
            "Failed to create replica config directory {}.",
            replica_config_dir.display()
        )
    })?;

    let replica_port_path = empty_writable_path(local_server_descriptor.replica_port_path())?;

    if background {
        send_background()?;
        return Runtime::new()
            .expect("Unable to create a runtime")
            .block_on(async {
                fg_ping_and_wait(
                    &webserver_port_path,
                    &frontend_url,
                    env.get_logger(),
                    local_server_descriptor,
                )
                .await
            });
    }
    local_server_descriptor.describe(env.get_logger());

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
        local_server_descriptor,
        &btc_adapter_config_path,
        &btc_adapter_socket_holder_path,
    )?;
    let btc_adapter_socket_path = btc_adapter_config
        .as_ref()
        .and_then(|cfg| cfg.get_socket_path());

    let canister_http_adapter_config = configure_canister_http_adapter_if_enabled(
        local_server_descriptor,
        &canister_http_adapter_config_path,
        &canister_http_adapter_socket_holder_path,
    )?;
    let canister_http_socket_path = canister_http_adapter_config
        .as_ref()
        .and_then(|cfg| cfg.get_socket_path());
    let subnet_type = local_server_descriptor
        .replica
        .subnet_type
        .unwrap_or_default();
    let log_level = local_server_descriptor
        .replica
        .log_level
        .unwrap_or_default();

    let proxy_domains = local_server_descriptor.proxy.domain.clone().into_vec();

    let replica_config = {
        let replica_config = ReplicaConfig::new(
            &state_root,
            subnet_type,
            log_level,
            artificial_delay,
            use_old_metering,
        );
        let mut replica_config = if let Some(port) = local_server_descriptor.replica.port {
            replica_config.with_port(port)
        } else {
            replica_config.with_random_port(&replica_port_path)
        };
        if let Some(btc_adapter_config) = btc_adapter_config.as_ref() {
            replica_config = replica_config.with_btc_adapter_enabled();
            if let Some(btc_adapter_socket) = btc_adapter_config.get_socket_path() {
                replica_config = replica_config.with_btc_adapter_socket(btc_adapter_socket);
            }
        }
        if let Some(canister_http_adapter_config) = canister_http_adapter_config.as_ref() {
            replica_config = replica_config.with_canister_http_adapter_enabled();
            if let Some(socket_path) = canister_http_adapter_config.get_socket_path() {
                replica_config = replica_config.with_canister_http_adapter_socket(socket_path);
            }
        }
        replica_config
    };

    let effective_config = CachedConfig::replica(&replica_config);

    if !clean && !force && previous_config_path.exists() {
        let previous_config = load_json_file(&previous_config_path)
            .context("Failed to read replica configuration. Rerun with `--clean`.")?;
        if effective_config != previous_config {
            bail!("The network configuration was changed. Rerun with `--clean`.")
        }
    }
    save_json_file(&previous_config_path, &effective_config)
        .context("Failed to write replica configuration")?;

    let network_descriptor = network_descriptor.clone();

    let system = actix::System::new();
    let _proxy = system.block_on(async move {
        let shutdown_controller = start_shutdown_controller(env)?;

        let port_ready_subscribe: Recipient<PortReadySubscribe> = {
            let btc_adapter_ready_subscribe = btc_adapter_config
                .map(|btc_adapter_config| {
                    start_btc_adapter_actor(
                        env,
                        btc_adapter_config_path,
                        btc_adapter_config.get_socket_path(),
                        shutdown_controller.clone(),
                        btc_adapter_pid_file_path,
                    )
                })
                .transpose()?;
            let canister_http_adapter_ready_subscribe = canister_http_adapter_config
                .map(|canister_http_adapter_config| {
                    start_canister_http_adapter_actor(
                        env,
                        canister_http_adapter_config_path,
                        canister_http_adapter_config.get_socket_path(),
                        shutdown_controller.clone(),
                        canister_http_adapter_pid_file_path,
                    )
                })
                .transpose()?;

            let replica = start_replica_actor(
                env,
                replica_config,
                local_server_descriptor,
                shutdown_controller.clone(),
                btc_adapter_ready_subscribe,
                canister_http_adapter_ready_subscribe,
            )?;
            replica.recipient()
        };

        let icx_proxy_config = IcxProxyConfig {
            bind: address_and_port,
            replica_urls: vec![], // will be determined after replica starts
            fetch_root_key: !network_descriptor.is_ic,
            domains: proxy_domains,
            verbose: env.get_verbose_level() > 0,
        };

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
        let _ = std::fs::remove_file(btc_adapter_socket_path);
    }
    if let Some(canister_http_socket_path) = canister_http_socket_path {
        let _ = std::fs::remove_file(canister_http_socket_path);
    }

    Ok(())
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum CachedReplicaConfig<'a> {
    Replica { config: Cow<'a, ReplicaConfig> },
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedConfig<'a> {
    pub replica_rev: String,
    #[serde(flatten)]
    pub config: CachedReplicaConfig<'a>,
}

impl<'a> CachedConfig<'a> {
    pub fn replica(config: &'a ReplicaConfig) -> Self {
        Self {
            replica_rev: replica_rev().into(),
            config: CachedReplicaConfig::Replica {
                config: Cow::Borrowed(config),
            },
        }
    }
}

pub fn apply_command_line_parameters(
    logger: &Logger,
    network_descriptor: NetworkDescriptor,
    host: Option<String>,
    replica_port: Option<String>,
    enable_bitcoin: bool,
    bitcoin_nodes: Vec<SocketAddr>,
    enable_canister_http: bool,
    domain: Vec<String>,
) -> DfxResult<NetworkDescriptor> {
    if enable_canister_http {
        warn!(
            logger,
            "The --enable-canister-http parameter is deprecated."
        );
        warn!(logger, "Canister HTTP suppport is enabled by default.  It can be disabled through dfx.json or networks.json.");
    }

    let _ = network_descriptor.local_server_descriptor()?;
    let mut local_server_descriptor = network_descriptor.local_server_descriptor.unwrap();

    if let Some(host) = host {
        let host: SocketAddr = host
            .parse()
            .map_err(|e| anyhow!("Invalid argument: Invalid host: {}", e))?;
        local_server_descriptor = local_server_descriptor.with_bind_address(host);
    }
    if let Some(replica_port) = replica_port {
        let replica_port: u16 = replica_port
            .parse()
            .map_err(|err| error_invalid_argument!("Invalid port number: {}", err))?;
        local_server_descriptor = local_server_descriptor.with_replica_port(replica_port);
    }
    if enable_bitcoin || !bitcoin_nodes.is_empty() {
        local_server_descriptor = local_server_descriptor.with_bitcoin_enabled();
    }

    if !bitcoin_nodes.is_empty() {
        local_server_descriptor = local_server_descriptor.with_bitcoin_nodes(bitcoin_nodes)
    }

    if !domain.is_empty() {
        local_server_descriptor = local_server_descriptor.with_proxy_domains(domain)
    }

    Ok(NetworkDescriptor {
        local_server_descriptor: Some(local_server_descriptor),
        ..network_descriptor
    })
}

#[context("Failed to clean existing replica state.")]
fn clean_state(
    local_server_descriptor: &LocalServerDescriptor,
    temp_dir: Option<PathBuf>,
) -> DfxResult {
    if local_server_descriptor.data_directory.is_dir() {
        fs::remove_dir_all(&local_server_descriptor.data_directory).with_context(|| {
            format!(
                "Cannot remove directory at '{}'",
                local_server_descriptor.data_directory.display()
            )
        })?;
    }

    if let Some(temp_dir) = temp_dir {
        let local_dir = temp_dir.join("local");
        if local_dir.is_dir() {
            fs::remove_dir_all(&local_dir).with_context(|| {
                format!("Cannot remove directory at '{}'.", local_dir.display())
            })?;
        }
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
    cmd.args(
        std::env::args()
            .skip(1)
            .filter(|a| !a.eq("--background"))
            .filter(|a| !a.eq("--clean")),
    );

    cmd.spawn().context("Failed to spawn child process.")?;
    Ok(())
}

#[context("Failed to get frontend address.")]
fn frontend_address(
    local_server_descriptor: &LocalServerDescriptor,
    background: bool,
) -> DfxResult<(String, SocketAddr)> {
    let mut address_and_port = local_server_descriptor.bind_address;

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

fn check_previous_process_running(
    local_server_descriptor: &LocalServerDescriptor,
) -> DfxResult<()> {
    for pid_path in local_server_descriptor.dfx_pid_paths() {
        if pid_path.exists() {
            // Read and verify it's not running. If it is just return.
            if let Ok(s) = std::fs::read_to_string(&pid_path) {
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
    }
    Ok(())
}

fn write_pid(pid_file_path: &Path) {
    if let Ok(pid) = sysinfo::get_current_pid() {
        let _ = std::fs::write(pid_file_path, pid.to_string());
    }
}

#[context("Failed to configure btc adapter.")]
pub fn configure_btc_adapter_if_enabled(
    local_server_descriptor: &LocalServerDescriptor,
    config_path: &Path,
    uds_holder_path: &Path,
) -> DfxResult<Option<bitcoin_adapter::Config>> {
    if !local_server_descriptor.bitcoin.enabled {
        return Ok(None);
    };

    let log_level = local_server_descriptor.bitcoin.log_level;

    let nodes = if let Some(ref nodes) = local_server_descriptor.bitcoin.nodes {
        nodes.clone()
    } else {
        bitcoin_adapter::default_nodes()
    };

    let config = write_btc_adapter_config(uds_holder_path, config_path, nodes, log_level)?;
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
    let uds_path = std::env::temp_dir().join(format!("{}.{}.{}", prefix, pid, timestamp_seconds));
    std::fs::write(uds_holder_path, uds_path.to_raw_bytes()).with_context(|| {
        format!(
            "unable to write unix domain socket path to {}",
            uds_holder_path.to_string_lossy()
        )
    })?;
    Ok(uds_path)
}

#[context("Failed to get persistent socket path for {} at {}.", prefix, uds_holder_path.to_string_lossy())]
fn get_persistent_socket_path(uds_holder_path: &Path, prefix: &str) -> DfxResult<PathBuf> {
    if let Ok(uds_path) = std::fs::read(uds_holder_path) {
        Ok(PathBuf::assert_from_raw_vec(uds_path))
    } else {
        create_new_persistent_socket_path(uds_holder_path, prefix)
    }
}

#[context("Failed to write btc adapter config to {}.", config_path.to_string_lossy())]
fn write_btc_adapter_config(
    uds_holder_path: &Path,
    config_path: &Path,
    nodes: Vec<SocketAddr>,
    log_level: bitcoin_adapter::BitcoinAdapterLogLevel,
) -> DfxResult<bitcoin_adapter::Config> {
    let socket_path = get_persistent_socket_path(uds_holder_path, "ic-btc-adapter-socket")?;

    let adapter_config = bitcoin_adapter::Config::new(nodes, socket_path, log_level);

    let contents = serde_json::to_string_pretty(&adapter_config)
        .context("Unable to serialize btc adapter configuration to json")?;
    std::fs::write(config_path, contents).with_context(|| {
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
    local_server_descriptor: &LocalServerDescriptor,
    config_path: &Path,
    uds_holder_path: &Path,
) -> DfxResult<Option<canister_http_adapter::Config>> {
    if !local_server_descriptor.canister_http.enabled {
        return Ok(None);
    };

    let socket_path =
        get_persistent_socket_path(uds_holder_path, "ic-https-outcalls-adapter-socket")?;

    let log_level = local_server_descriptor.canister_http.log_level;
    let adapter_config = canister_http_adapter::Config::new(socket_path, log_level);

    let contents = serde_json::to_string_pretty(&adapter_config)
        .context("Unable to serialize canister http adapter configuration to json")?;
    std::fs::write(config_path, contents)
        .with_context(|| format!("Unable to write {}", config_path.to_string_lossy()))?;

    Ok(Some(adapter_config))
}
