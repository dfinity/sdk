use crate::actors::pocketic::PocketIcProxyConfig;
use crate::actors::{start_pocketic_actor, start_post_start_actor, start_shutdown_controller};
use crate::config::dfx_version_str;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::info::replica_rev;
use crate::lib::network::id::write_network_id;
use crate::lib::replica::status::ping_and_wait;
use crate::util::get_reusable_socket_addr;
use anyhow::{Context, Error, anyhow, bail, ensure};
use clap::{ArgAction, Parser};
use dfx_core::{
    config::model::{
        local_server_descriptor::{LocalNetworkScopeDescriptor, LocalServerDescriptor},
        network_descriptor::NetworkDescriptor,
        replica_config::{CachedConfig, ReplicaConfig},
        settings_digest::get_settings_digest,
    },
    fs,
    json::{load_json_file, save_json_file},
    network::provider::{LocalBindDetermination, create_network_descriptor},
};
use fn_error_context::context;
use slog::{Logger, info, warn};
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
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

    /// Indicates if the actual dfx process is running in the background.
    #[arg(long, env = "DFX_RUNNING_IN_BACKGROUND", hide = true)]
    running_in_background: bool,

    /// Cleans the state of the current project.
    #[arg(long)]
    clean: bool,

    /// Bootstraps system canisters.
    #[arg(long)]
    system_canisters: bool,

    /// Address of bitcoind node.  Implies --enable-bitcoin.
    #[arg(long, action = ArgAction::Append)]
    bitcoin_node: Vec<SocketAddr>,

    /// enable bitcoin integration. If --bitcoin_node is not passed, defaults to 127.0.0.1:18444
    #[arg(long)]
    enable_bitcoin: bool,

    /// Address of dogecoind node.  Implies --enable-dogecoin.
    #[arg(long, action = ArgAction::Append)]
    dogecoin_node: Vec<SocketAddr>,

    /// enable dogecoin integration. If --dogecoin_node is not passed, defaults to 127.0.0.1:18444
    #[arg(long)]
    enable_dogecoin: bool,

    /// enable canister http requests (on by default)
    #[arg(long)]
    enable_canister_http: bool,

    /// The delay (in milliseconds) an update call should take. Lower values may be expedient in CI.
    #[arg(long, default_value_t = 600)]
    artificial_delay: u32,

    /// Start even if the network config was modified.
    #[arg(long)]
    force: bool,

    /// A list of domains that can be served. These are used for canister resolution [default: localhost]
    #[arg(long)]
    domain: Vec<String>,

    /// Runs PocketIC instead of the replica
    /// Currently this has no effect.
    #[clap(long, alias = "emulator")]
    #[allow(unused)]
    pocketic: bool,

    #[clap(long, hide = true)]
    replica: bool,
}

// The frontend webserver is brought up by the bg process; thus, the fg process
// needs to wait and verify it's up before exiting.
// Because the user may have specified to start on port 0, here we wait for
// webserver_port_path to get written to and modify the frontend_url so we
// ping the correct address.
async fn fg_ping_and_wait(
    pocketic_port_path: &Path,
    webserver_port_path: &Path,
    frontend_url: &str,
) -> DfxResult {
    let port = wait_for_port(webserver_port_path).await?;
    _ = wait_for_port(pocketic_port_path).await?; // used as a signal that initialization is complete
    // not needed for network functionality, but ensures the child is done sending to stderr

    let mut frontend_url_mod = frontend_url.to_string();
    let port_offset = frontend_url_mod
        .as_str()
        .rfind(':')
        .ok_or_else(|| anyhow!("Malformed frontend url: {}", frontend_url))?;
    frontend_url_mod.replace_range((port_offset + 1).., port.as_str());
    ping_and_wait(&frontend_url_mod).await
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
        running_in_background,
        clean,
        system_canisters,
        force,
        bitcoin_node,
        enable_bitcoin,
        dogecoin_node,
        enable_dogecoin,
        enable_canister_http,
        artificial_delay,
        domain,
        pocketic: _,
        replica,
    }: StartOpts,
) -> DfxResult {
    ensure!(!replica, "The 'native' replica (--replica) is no longer supported. See the 0.27.0 migration guide for more information.
https://github.com/dfinity/sdk/blob/0.27.0/docs/migration/dfx-0.27.0-migration-guide.md");
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
        enable_bitcoin,
        bitcoin_node,
        enable_dogecoin,
        dogecoin_node,
        enable_canister_http,
        domain,
        artificial_delay,
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

    fs::create_dir_all(&local_server_descriptor.data_dir_by_settings_digest())?;

    if !local_server_descriptor.network_id_path().exists() {
        write_network_id(local_server_descriptor)?;
    }
    if let LocalNetworkScopeDescriptor::Shared { network_id_path } = &local_server_descriptor.scope
    {
        fs::copy(&local_server_descriptor.network_id_path(), network_id_path)?;
        let effective_config_path_by_settings_digest =
            local_server_descriptor.effective_config_path_by_settings_digest();
        if effective_config_path_by_settings_digest.exists() {
            fs::copy(
                &effective_config_path_by_settings_digest,
                &local_server_descriptor.effective_config_path(),
            )?;
        }
    }

    clean_older_state_dirs(local_server_descriptor)?;
    let state_root = local_server_descriptor.state_dir();
    let pid_file_path = empty_writable_path(pid_file_path)?;
    let webserver_port_path = empty_writable_path(local_server_descriptor.webserver_port_path())?;

    let previous_config_path = local_server_descriptor.effective_config_path();

    let pocketic_port_path = empty_writable_path(local_server_descriptor.pocketic_port_path())?;

    if background {
        send_background()?;
        return Runtime::new()
            .expect("Unable to create a runtime")
            .block_on(async {
                fg_ping_and_wait(&pocketic_port_path, &webserver_port_path, &frontend_url).await
            });
    }
    local_server_descriptor.describe(env.get_logger());

    write_pid(&pid_file_path);
    fs::write(&webserver_port_path, address_and_port.port().to_string())?;

    let subnet_type = local_server_descriptor
        .replica
        .subnet_type
        .unwrap_or_default();
    let log_level = local_server_descriptor
        .replica
        .log_level
        .unwrap_or_default();

    let proxy_domains = local_server_descriptor
        .proxy
        .domain
        .clone()
        .map(|v| v.into_vec());

    let replica_config = {
        let mut replica_config =
            ReplicaConfig::new(&state_root, subnet_type, log_level, artificial_delay);
        if local_server_descriptor.bitcoin.enabled {
            replica_config = replica_config.with_btc_adapter_enabled();
        }
        if local_server_descriptor.canister_http.enabled {
            replica_config = replica_config.with_canister_http_adapter_enabled();
        }
        if system_canisters {
            replica_config = replica_config.with_system_canisters();
        }
        replica_config
    };

    let effective_config = if replica {
        CachedConfig::replica(&replica_config, replica_rev().into())
    } else {
        CachedConfig::pocketic(&replica_config, replica_rev().into(), None)
    };

    let is_shared_network = matches!(
        &local_server_descriptor.scope,
        LocalNetworkScopeDescriptor::Shared { .. }
    );
    if is_shared_network {
        save_json_file(
            &local_server_descriptor.effective_config_path_by_settings_digest(),
            &effective_config,
        )?;
    } else if !clean && !force && previous_config_path.exists() {
        let previous_config = load_json_file(&previous_config_path)
            .context("Failed to read replica configuration. Rerun with `--clean`.")?;
        if !effective_config.can_share_state(&previous_config) {
            bail!(
                "The network state can't be reused with this configuration. Rerun with `--clean`."
            )
        }
    }
    save_json_file(&previous_config_path, &effective_config)?;

    let spinner = env.new_spinner("Starting local network...".into());
    let system = actix::System::new();
    let _post_start = system.block_on(async move {
        let shutdown_controller = start_shutdown_controller(env)?;

        let pocketic_proxy_config = PocketIcProxyConfig {
            bind: address_and_port,
            domains: proxy_domains,
        };

        let server = start_pocketic_actor(
            env,
            replica_config,
            local_server_descriptor,
            shutdown_controller.clone(),
            pocketic_port_path,
            pocketic_proxy_config,
        )?;

        let post_start = start_post_start_actor(env, running_in_background, Some(server), spinner)?;

        Ok::<_, Error>(post_start)
    })?;
    system.run()?;
    Ok(())
}

fn clean_older_state_dirs(local_server_descriptor: &LocalServerDescriptor) -> DfxResult {
    let directories_to_keep = 10;
    let settings_digest = local_server_descriptor.settings_digest.as_ref().unwrap();

    let data_dir = &local_server_descriptor.data_directory;
    if !data_dir.is_dir() {
        return Ok(());
    }
    let mut state_dirs = fs::read_dir(data_dir)?
        .filter_map(|e| match e {
            Ok(entry) if is_candidate_state_dir(&entry.path(), settings_digest) => {
                Some(Ok(entry.path()))
            }
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    // keep the X most recent directories
    state_dirs.sort_by_cached_key(|p| {
        p.metadata()
            .map(|m| m.modified().unwrap_or(SystemTime::UNIX_EPOCH))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });
    state_dirs = state_dirs
        .iter()
        .rev()
        .skip(directories_to_keep)
        .cloned()
        .collect();

    for dir in state_dirs {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

fn is_candidate_state_dir(path: &Path, settings_digest: &str) -> bool {
    path.is_dir()
        && path
            .file_name()
            .map(|f| {
                let filename: String = f.to_string_lossy().into();
                filename != *settings_digest
            })
            .unwrap_or(true)
}

pub fn apply_command_line_parameters(
    logger: &Logger,
    network_descriptor: NetworkDescriptor,
    host: Option<String>,
    enable_bitcoin: bool,
    bitcoin_nodes: Vec<SocketAddr>,
    enable_dogecoin: bool,
    dogecoin_nodes: Vec<SocketAddr>,
    enable_canister_http: bool,
    domain: Vec<String>,
    artificial_delay: u32,
) -> DfxResult<NetworkDescriptor> {
    if enable_canister_http {
        warn!(
            logger,
            "The --enable-canister-http parameter is deprecated."
        );
        warn!(
            logger,
            "Canister HTTP suppport is enabled by default.  It can be disabled through dfx.json or networks.json."
        );
    }

    let _ = network_descriptor.local_server_descriptor()?;
    let mut local_server_descriptor = network_descriptor.local_server_descriptor.unwrap();

    if let Some(host) = host {
        let host: SocketAddr = host
            .parse()
            .map_err(|e| anyhow!("Invalid argument: Invalid host: {}", e))?;
        local_server_descriptor = local_server_descriptor.with_bind_address(host);
    }
    if enable_bitcoin || !bitcoin_nodes.is_empty() {
        local_server_descriptor = local_server_descriptor.with_bitcoin_enabled();
    }

    if !bitcoin_nodes.is_empty() {
        local_server_descriptor = local_server_descriptor.with_bitcoin_nodes(bitcoin_nodes)
    }

    if enable_dogecoin || !dogecoin_nodes.is_empty() {
        local_server_descriptor = local_server_descriptor.with_dogecoin_enabled();
    }

    if !dogecoin_nodes.is_empty() {
        local_server_descriptor = local_server_descriptor.with_dogecoin_nodes(dogecoin_nodes)
    }

    if !domain.is_empty() {
        local_server_descriptor = local_server_descriptor.with_proxy_domains(domain)
    }

    let settings_digest =
        get_settings_digest(replica_rev(), &local_server_descriptor, artificial_delay);

    local_server_descriptor = local_server_descriptor.with_settings_digest(settings_digest);

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
    )
    .env("DFX_RUNNING_IN_BACKGROUND", "true"); // Set the `DFX_RUNNING_IN_BACKGROUND` environment variable which will be used by the second start.

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

pub fn empty_writable_path(path: PathBuf) -> DfxResult<PathBuf> {
    std::fs::write(&path, "")
        .with_context(|| format!("Unable to write to {}", path.to_string_lossy()))?;
    Ok(path)
}
