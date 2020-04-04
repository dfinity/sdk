use crate::config::dfinity::ConfigDefaultsBootstrap;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::webserver::webserver;

use slog::{info, trace};
use std::default::Default;
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use url::Url;

/// Runs the bootstrap server.
pub fn exec(env: &dyn Environment, args: &ConfigDefaultsBootstrap) -> DfxResult {
    let logger = env.get_logger();
    let config = get_config(args, env)?;
    trace!(logger, "config = {:?}", config);
    let ip = config.ip.unwrap();
    let port = config.port.unwrap();
    let address = SocketAddr::new(ip, port);
    let providers = config.providers.into_iter().collect();
    let root = config.root.unwrap();
    let (sender, receiver) = crossbeam::unbounded();
    webserver(logger.clone(), address, providers, &root, sender)?
        .join()
        .map_err(|err| DfxError::Io(Error::new(ErrorKind::Other, format!("{:?}", err))))?;
    let _ = receiver
        .recv()
        .expect("Failed to receive signal from bootstrap server!");
    info!(logger, "Bootstrap server is ready...");
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Gets the configuration options for the bootstrap server. Each option is checked for correctness
/// and otherwise guaranteed to exist.
fn get_config(
    args: &ConfigDefaultsBootstrap,
    env: &dyn Environment,
) -> DfxResult<ConfigDefaultsBootstrap> {
    let base = get_config_from_file(env);
    let ip = get_ip(args, &base);
    let port = get_port(args, &base);
    let providers = get_providers(args, &base);
    let root = get_root(args, &base, env)?;
    let timeout = get_timeout(args, &base);
    Ok(ConfigDefaultsBootstrap {
        ip: Some(ip),
        port: Some(port),
        providers: providers,
        root: Some(root),
        timeout: Some(timeout),
    })
}

/// Gets the configuration options for the bootstrap server as they were specified in the dfx
/// configuration file.
fn get_config_from_file(env: &dyn Environment) -> ConfigDefaultsBootstrap {
    env.get_config().map_or(Default::default(), |config| {
        config
            .get_config()
            .get_defaults()
            .get_bootstrap()
            .to_owned()
    })
}

/// Gets the IP address that the bootstrap server listens on. First checks if the IP address was
/// specified on the command-line using --ip, otherwise checks if the IP address was specified in
/// the dfx configuration file, otherise defaults to 127.0.0.1.
fn get_ip(args: &ConfigDefaultsBootstrap, base: &ConfigDefaultsBootstrap) -> IpAddr {
    let default = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    args.ip.unwrap_or(base.ip.unwrap_or(default))
}

/// Gets the port number that the bootstrap server listens on. First checks if the port number was
/// specified on the command-line using --port, otherwise checks if the port number was specified
/// in the dfx configuration file, otherise defaults to 8081.
fn get_port(args: &ConfigDefaultsBootstrap, base: &ConfigDefaultsBootstrap) -> u16 {
    let default = 8081;
    args.port.unwrap_or(base.port.unwrap_or(default))
}

/// Gets the list of compute provider API endpoints. First checks if the providers were specified
/// on the command-line using --providers, otherwise checks if the providers were specified in the
/// dfx configuration file, otherwise defaults to http://127.0.0.1:8080/api.
fn get_providers(args: &ConfigDefaultsBootstrap, base: &ConfigDefaultsBootstrap) -> Vec<Url> {
    let default = vec![Url::parse("http://127.0.0.1:8080/api").unwrap()];
    if !args.providers.is_empty() {
        args.providers.clone()
    } else if !base.providers.is_empty() {
        base.providers.clone()
    } else {
        default
    }
}

/// Gets the directory containing static assets served by the bootstrap server. First checks if the
/// directory was specified on the command-line using --root, otherwise checks if the directory was
/// specified in the dfx configuration file, otherise defaults to
/// $HOME/.cache/dfinity/versions/$DFX_VERSION/js-user-library/dist/bootstrap.
fn get_root(
    args: &ConfigDefaultsBootstrap,
    base: &ConfigDefaultsBootstrap,
    env: &dyn Environment,
) -> DfxResult<PathBuf> {
    let default = env
        .get_cache()
        .get_binary_command_path("js-user-library/dist/bootstrap")?;
    Ok(args
        .root
        .clone()
        .unwrap_or(base.root.clone().unwrap_or(default)))
}

/// Gets the maximum amount of time, in seconds, the bootstrap server will wait for upstream
/// requests to complete. First checks if the timeout was specified on the command-line using
/// --timeout, otherwise checks if the timeout was specified in the dfx configuration file,
/// otherise defaults to 30.
fn get_timeout(args: &ConfigDefaultsBootstrap, base: &ConfigDefaultsBootstrap) -> u64 {
    let default = 30;
    args.timeout.unwrap_or(base.timeout.unwrap_or(default))
}
