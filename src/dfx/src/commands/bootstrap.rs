use crate::config::dfinity::{ConfigDefaults, ConfigDefaultsBootstrap};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::get_network_descriptor;
use crate::util::get_reusable_socket_addr;

use crate::actors::icx_proxy::signals::PortReadySubscribe;
use crate::actors::icx_proxy::IcxProxyConfig;
use crate::actors::{start_icx_proxy_actor, start_shutdown_controller};
use crate::commands::start::start_webserver_coordinator;
use actix::Recipient;
use anyhow::Context;
use clap::Clap;
use std::default::Default;
use std::net::{IpAddr, Ipv4Addr};

/// Starts the bootstrap server.
#[derive(Clap, Clone)]
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
    // let logger = env.get_logger();
    let config = env.get_config_or_anyhow()?;
    let config_defaults = get_config_defaults_from_file(env);
    let base_config_bootstrap = config_defaults.get_bootstrap().to_owned();
    let config_bootstrap = apply_arguments(&base_config_bootstrap, env, opts.clone())?;

    let network_descriptor = get_network_descriptor(env, opts.network)?;
    let build_output_root = config.get_temp_path().join(network_descriptor.name.clone());
    let build_output_root = build_output_root.join("canisters");
    let icx_proxy_pid_file_path = env.get_temp_dir().join("icx-proxy-pid");

    // let providers = get_providers(&network_descriptor)?;

    // Since the user may have provided port "0", we need to grab a dynamically
    // allocated port and construct a resuable SocketAddr which the actix
    // HttpServer will bind to
    let socket_addr =
        get_reusable_socket_addr(config_bootstrap.ip.unwrap(), config_bootstrap.port.unwrap())?;

    let webserver_port_path = env.get_temp_dir().join("webserver-port");
    std::fs::write(&webserver_port_path, "")?;
    std::fs::write(&webserver_port_path, socket_addr.port().to_string())?;

    let system = actix::System::new("dfx-bootstrap");

    let shutdown_controller = start_shutdown_controller(env)?;

    let port_ready_subscribe: Option<Recipient<PortReadySubscribe>> = None;

    let webserver_bind = get_reusable_socket_addr(socket_addr.ip(), 0)?;

    let _webserver_coordinator = start_webserver_coordinator(
        env,
        network_descriptor,
        webserver_bind,
        build_output_root,
        port_ready_subscribe.clone(),
        shutdown_controller.clone(),
    )?;

    let icx_proxy_config = IcxProxyConfig {
        bind: socket_addr,
        candid_port: webserver_bind.port(),
    };
    let _proxy = start_icx_proxy_actor(
        env,
        icx_proxy_config,
        port_ready_subscribe,
        shutdown_controller,
        icx_proxy_pid_file_path,
    )?;
    system.run()?;

    Ok(())
}

/// Gets the configuration options for the bootstrap server. Each option is checked for correctness
/// and otherwise guaranteed to exist.
fn apply_arguments(
    config: &ConfigDefaultsBootstrap,
    _env: &dyn Environment,
    opts: BootstrapOpts,
) -> DfxResult<ConfigDefaultsBootstrap> {
    let ip = get_ip(&config, opts.ip.as_deref())?;
    let port = get_port(&config, opts.port.as_deref())?;
    let timeout = get_timeout(&config, opts.timeout.as_deref())?;
    Ok(ConfigDefaultsBootstrap {
        ip: Some(ip),
        port: Some(port),
        timeout: Some(timeout),
    })
}

/// Gets the configuration options for the bootstrap server as they were specified in the dfx
/// configuration file.
fn get_config_defaults_from_file(env: &dyn Environment) -> ConfigDefaults {
    env.get_config().map_or(Default::default(), |config| {
        config.get_config().get_defaults().to_owned()
    })
}

/// Gets the IP address that the bootstrap server listens on. First checks if the IP address was
/// specified on the command-line using --ip, otherwise checks if the IP address was specified in
/// the dfx configuration file, otherise defaults to 127.0.0.1.
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
fn get_port(config: &ConfigDefaultsBootstrap, port: Option<&str>) -> DfxResult<u16> {
    port.map(|port| port.parse())
        .unwrap_or_else(|| {
            let default = 8081;
            Ok(config.port.unwrap_or(default))
        })
        .context("Invalid argument: Invalid port number.")
}

/// Gets the list of compute provider API endpoints.
fn _get_providers(network_descriptor: &NetworkDescriptor) -> DfxResult<Vec<String>> {
    network_descriptor
        .providers
        .iter()
        .map(|url| Ok(format!("{}/api", url)))
        .collect()
}

/// Gets the maximum amount of time, in seconds, the bootstrap server will wait for upstream
/// requests to complete. First checks if the timeout was specified on the command-line using
/// --timeout, otherwise checks if the timeout was specified in the dfx configuration file,
/// otherise defaults to 30.
fn get_timeout(config: &ConfigDefaultsBootstrap, timeout: Option<&str>) -> DfxResult<u64> {
    timeout
        .map(|timeout| timeout.parse())
        .unwrap_or_else(|| {
            let default = 30;
            Ok(config.timeout.unwrap_or(default))
        })
        .context("Invalid argument: Invalid timeout.")
}
