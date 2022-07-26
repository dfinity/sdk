use crate::actors::icx_proxy::IcxProxyConfig;
use crate::actors::{start_icx_proxy_actor, start_shutdown_controller};
use crate::config::dfinity::ConfigDefaultsBootstrap;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::{get_network_descriptor, LocalBindDetermination};
use crate::lib::webserver::run_webserver;
use crate::util::get_reusable_socket_addr;

use anyhow::{anyhow, Context, Error};
use clap::Parser;
use fn_error_context::context;
use slog::info;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use url::Url;

/// Starts the bootstrap server.
#[derive(Parser, Clone)]
pub struct BootstrapOpts {
    /// Specifies the IP address that the bootstrap server listens on. Defaults to 127.0.0.1.
    #[clap(long)]
    ip: Option<String>,

    /// Specifies the port number that the bootstrap server listens on. Defaults to 8081.
    #[clap(long)]
    port: Option<String>,

    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[clap(long)]
    network: Option<String>,

    /// Specifies the directory containing static assets served by the bootstrap server.
    /// Defaults to $HOME/.cache/dfinity/versions/$DFX_VERSION/js-user-library/dist/bootstrap.
    #[clap(long)]
    root: Option<String>,

    /// Specifies the maximum number of seconds that the bootstrap server
    /// will wait for upstream requests to complete. Defaults to 30.
    #[clap(long)]
    timeout: Option<String>,
}

/// Runs the bootstrap server.
pub fn exec(env: &dyn Environment, opts: BootstrapOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let network_descriptor = get_network_descriptor(
        env.get_config(),
        opts.network.clone(),
        LocalBindDetermination::AsConfigured,
    )?;
    let local_server_descriptor = network_descriptor.local_server_descriptor()?;
    let config_bootstrap = apply_arguments(&local_server_descriptor.bootstrap, opts)?;

    let build_output_root = config.get_temp_path().join(network_descriptor.name.clone());
    let build_output_root = build_output_root.join("canisters");
    let icx_proxy_pid_file_path = local_server_descriptor.icx_proxy_pid_path();
    let proxy_port_path = local_server_descriptor.proxy_port_path();

    let replica_urls = get_replica_urls(env, &network_descriptor)?;

    // Since the user may have provided port "0", we need to grab a dynamically
    // allocated port and construct a resuable SocketAddr which the actix
    // HttpServer will bind to
    let socket_addr =
        get_reusable_socket_addr(config_bootstrap.ip.unwrap(), config_bootstrap.port.unwrap())
            .context("Failed to find socket address for the HTTP server.")?;

    let webserver_port_path = local_server_descriptor.webserver_port_path();
    std::fs::write(&webserver_port_path, "").with_context(|| {
        format!(
            "Failed to write/clear webserver port file {}.",
            webserver_port_path.to_string_lossy()
        )
    })?;
    std::fs::write(&webserver_port_path, socket_addr.port().to_string()).with_context(|| {
        format!(
            "Failed to write port to webserver port file {}.",
            webserver_port_path.to_string_lossy()
        )
    })?;

    verify_unique_ports(&replica_urls, &socket_addr)?;

    let system = actix::System::new();
    let _proxy = system
        .block_on(async move {
            let shutdown_controller = start_shutdown_controller(env)?;

            let webserver_bind = get_reusable_socket_addr(socket_addr.ip(), 0)?;
            std::fs::write(&proxy_port_path, "").with_context(|| {
                format!(
                    "Failed to write/clear proxy port file {}.",
                    proxy_port_path.to_string_lossy()
                )
            })?;
            std::fs::write(&proxy_port_path, webserver_bind.port().to_string()).with_context(
                || {
                    format!(
                        "Failed to write port to proxy port file {}.",
                        proxy_port_path.to_string_lossy()
                    )
                },
            )?;

            let icx_proxy_config = IcxProxyConfig {
                bind: socket_addr,
                proxy_port: webserver_bind.port(),
                replica_urls,
                fetch_root_key: !network_descriptor.is_ic,
            };

            run_webserver(
                env.get_logger().clone(),
                build_output_root,
                network_descriptor,
                config,
                env.get_temp_dir().to_path_buf(),
                webserver_bind,
            )?;

            let port_ready_subscribe = None;
            let proxy = start_icx_proxy_actor(
                env,
                icx_proxy_config,
                port_ready_subscribe,
                shutdown_controller,
                icx_proxy_pid_file_path,
            )?;
            Ok::<_, Error>(proxy)
        })
        .context("Failed to start proxy.")?;
    system.run().context("Failed to run system runner.")?;

    Ok(())
}

#[context("Cannot bind to and serve from the same port.")]
fn verify_unique_ports(providers: &[url::Url], bind: &SocketAddr) -> DfxResult {
    // Verify that we cannot bind to a port that we forward to.
    let bound_port = bind.port();
    let bind_and_forward_on_same_port = providers.iter().any(|url| {
        Some(bound_port) == url.port()
            && match url.host_str() {
                Some(h) => h == "localhost" || h == "::1" || h == "127.0.0.1",
                None => true,
            }
    });
    if bind_and_forward_on_same_port {
        return Err(anyhow!(
            "Cannot forward API calls to the same bootstrap server."
        ));
    }
    Ok(())
}

/// Gets the configuration options for the bootstrap server. Each option is checked for correctness
/// and otherwise guaranteed to exist.
#[context("Failed to determine bootstrap server configuration.")]
fn apply_arguments(
    config: &ConfigDefaultsBootstrap,
    opts: BootstrapOpts,
) -> DfxResult<ConfigDefaultsBootstrap> {
    let ip = get_ip(config, opts.ip.as_deref())?;
    let port = get_port(config, opts.port.as_deref())?;
    let timeout = get_timeout(config, opts.timeout.as_deref())?;
    Ok(ConfigDefaultsBootstrap {
        ip: Some(ip),
        port: Some(port),
        timeout: Some(timeout),
    })
}

/// Gets the IP address that the bootstrap server listens on. First checks if the IP address was
/// specified on the command-line using --ip, otherwise checks if the IP address was specified in
/// the dfx configuration file, otherise defaults to 127.0.0.1.
#[context("Failed to get ip that the bootstrap server listens on.")]
fn get_ip(config: &ConfigDefaultsBootstrap, ip: Option<&str>) -> DfxResult<IpAddr> {
    ip.map(|ip| ip.parse())
        .unwrap_or_else(|| {
            let default = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
            Ok(config.ip.unwrap_or(default))
        })
        .context("Invalid argument: Invalid IP address.")
}

/// Gets the port number that the bootstrap server listens on. First checks if the port number was
/// specified on the command-line using --port, otherwise checks if the port number was specified
/// in the dfx configuration file, otherise defaults to 8081.

#[context("Failed to get port that the bootstrap server listens on.")]
fn get_port(config: &ConfigDefaultsBootstrap, port: Option<&str>) -> DfxResult<u16> {
    port.map(|port| port.parse())
        .unwrap_or_else(|| {
            let default = 8081;
            Ok(config.port.unwrap_or(default))
        })
        .context("Invalid argument: Invalid port number.")
}

#[context("Failed to determine replica urls.")]
fn get_replica_urls(
    env: &dyn Environment,
    network_descriptor: &NetworkDescriptor,
) -> DfxResult<Vec<Url>> {
    if network_descriptor.name == "local" {
        let local_server_descriptor = network_descriptor.local_server_descriptor()?;
        if let Some(port) = get_running_replica_port(env, local_server_descriptor)? {
            let mut socket_addr = local_server_descriptor.bind_address;
            socket_addr.set_port(port);
            let url = format!("http://{}", socket_addr);
            let url = Url::parse(&url)?;
            return Ok(vec![url]);
        }
    }
    get_providers(network_descriptor)
}

fn get_running_replica_port(
    env: &dyn Environment,
    local_server_descriptor: &LocalServerDescriptor,
) -> DfxResult<Option<u16>> {
    let logger = env.get_logger();
    // dfx start and dfx replica both write these as empty, and then
    // populate one with a port.
    let emulator_port_path = local_server_descriptor.ic_ref_port_path();
    let replica_port_path = local_server_descriptor.replica_port_path();

    match read_port_from(&replica_port_path)? {
        Some(port) => {
            info!(logger, "Found local replica running on port {}", port);
            Ok(Some(port))
        }
        None => match read_port_from(&emulator_port_path)? {
            Some(port) => {
                info!(logger, "Found local emulator running on port {}", port);
                Ok(Some(port))
            }
            None => Ok(None),
        },
    }
}

/// Gets the list of compute provider API endpoints.
#[context("Failed to get providers for network '{}'.", network_descriptor.name)]
fn get_providers(network_descriptor: &NetworkDescriptor) -> DfxResult<Vec<Url>> {
    network_descriptor
        .providers
        .iter()
        .map(|url| parse_url(url))
        .collect()
}

#[context("Failed to parse url '{}'.", url)]
fn parse_url(url: &str) -> DfxResult<Url> {
    Ok(Url::parse(url)?)
}

#[context("Failed to read port value from {}", path.to_string_lossy())]
fn read_port_from(path: &Path) -> DfxResult<Option<u16>> {
    if path.exists() {
        let s = std::fs::read_to_string(&path)?;
        let s = s.trim();
        if s.is_empty() {
            Ok(None)
        } else {
            let port = s.parse::<u16>()?;
            Ok(Some(port))
        }
    } else {
        Ok(None)
    }
}
/// Gets the maximum amount of time, in seconds, the bootstrap server will wait for upstream
/// requests to complete. First checks if the timeout was specified on the command-line using
/// --timeout, otherwise checks if the timeout was specified in the dfx configuration file,
/// otherise defaults to 30.
#[context("Failed to determine timeout for bootstrap server.")]
fn get_timeout(config: &ConfigDefaultsBootstrap, timeout: Option<&str>) -> DfxResult<u64> {
    timeout
        .map(|timeout| timeout.parse())
        .unwrap_or_else(|| {
            let default = 30;
            Ok(config.timeout.unwrap_or(default))
        })
        .context("Invalid argument: Invalid timeout.")
}
