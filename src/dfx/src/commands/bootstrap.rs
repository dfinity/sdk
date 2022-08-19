use crate::actors::icx_proxy::IcxProxyConfig;
use crate::actors::{start_icx_proxy_actor, start_shutdown_controller};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::{create_network_descriptor, LocalBindDetermination};
use crate::util::get_reusable_socket_addr;

use anyhow::{anyhow, Context, Error};
use clap::Parser;
use fn_error_context::context;
use slog::info;
use std::fs::create_dir_all;
use std::net::{IpAddr, SocketAddr};
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

    /// Specifies the maximum number of seconds that the bootstrap server
    /// will wait for upstream requests to complete. Defaults to 30.
    #[clap(long)]
    timeout: Option<String>,
}

/// Runs the bootstrap server.
pub fn exec(
    env: &dyn Environment,
    BootstrapOpts {
        ip,
        port,
        network,
        timeout,
    }: BootstrapOpts,
) -> DfxResult {
    let network_descriptor = create_network_descriptor(
        env.get_config(),
        env.get_networks_config(),
        network,
        Some(env.get_logger().clone()),
        LocalBindDetermination::AsConfigured,
    )?;
    let network_descriptor =
        apply_arguments(network_descriptor, ip, port.as_deref(), timeout.as_deref())?;
    let local_server_descriptor = network_descriptor.local_server_descriptor()?;
    local_server_descriptor.describe_bootstrap();
    let config_bootstrap = &local_server_descriptor.bootstrap;

    create_dir_all(&local_server_descriptor.data_directory).with_context(|| {
        format!(
            "Failed to create network temp directory {}.",
            local_server_descriptor.data_directory.to_string_lossy()
        )
    })?;

    let icx_proxy_pid_file_path = local_server_descriptor.icx_proxy_pid_path();

    let replica_urls = get_replica_urls(env, &network_descriptor)?;

    // Since the user may have provided port "0", we need to grab a dynamically
    // allocated port and construct a resuable SocketAddr which the actix
    // HttpServer will bind to
    let socket_addr = get_reusable_socket_addr(config_bootstrap.ip, config_bootstrap.port)
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

            let icx_proxy_config = IcxProxyConfig {
                bind: socket_addr,
                replica_urls,
                fetch_root_key: !network_descriptor.is_ic,
            };

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

#[context("Failed to determine bootstrap server configuration.")]
fn apply_arguments(
    network_descriptor: NetworkDescriptor,
    ip: Option<String>,
    port: Option<&str>,
    timeout: Option<&str>,
) -> DfxResult<NetworkDescriptor> {
    let _ = network_descriptor.local_server_descriptor()?;
    let mut local_server_descriptor = network_descriptor.local_server_descriptor.unwrap();

    if let Some(ip) = ip {
        let ip: IpAddr = ip
            .parse()
            .context("Invalid argument: Invalid IP address.")?;
        local_server_descriptor = local_server_descriptor.with_bootstrap_ip(ip);
    }

    if let Some(port) = port {
        let port: u16 = port
            .parse()
            .context("Invalid argument: Invalid port number.")?;
        local_server_descriptor = local_server_descriptor.with_bootstrap_port(port);
    }

    if let Some(timeout) = timeout {
        let timeout: u64 = timeout
            .parse()
            .context("Invalid argument: Invalid timeout.")?;
        local_server_descriptor = local_server_descriptor.with_bootstrap_timeout(timeout);
    }

    Ok(NetworkDescriptor {
        local_server_descriptor: Some(local_server_descriptor),
        ..network_descriptor
    })
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
