use crate::config::dfinity::{ConfigDefaults, ConfigDefaultsBootstrap};
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::provider::{get_network_context, get_network_descriptor};
use crate::lib::webserver::webserver;
use clap::{App, Arg, ArgMatches, SubCommand};
use slog::info;
use std::default::Default;
use std::fs;
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

/// Constructs a sub-command to run the bootstrap server.
pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("bootstrap")
        .about(UserMessage::BootstrapCommand.to_str())
        .arg(
            Arg::with_name("ip")
                .help(UserMessage::BootstrapIP.to_str())
                .long("ip")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .help(UserMessage::BootstrapPort.to_str())
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("network")
                .help(UserMessage::CanisterComputeNetwork.to_str())
                .long("network")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("root")
                .help(UserMessage::BootstrapRoot.to_str())
                .long("root")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("timeout")
                .help(UserMessage::BootstrapTimeout.to_str())
                .long("timeout")
                .takes_value(true),
        )
}

/// Runs the bootstrap server.
pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let logger = env.get_logger();
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let config_defaults = get_config_defaults_from_file(env);
    let base_config_bootstrap = config_defaults.get_bootstrap().to_owned();
    let config_bootstrap = apply_arguments(&base_config_bootstrap, env, args)?;

    let network_descriptor = get_network_descriptor(env, args)?;

    let network_name = get_network_context().ok_or_else(|| DfxError::ComputeNetworkNotSet)?;

    let build_root = config_defaults.get_build().get_output();

    let build_output_root = config.get_temp_path().join(network_name);
    let build_output_root = build_output_root.join(build_root);

    let providers = get_providers(&network_descriptor)?;

    let (sender, receiver) = crossbeam::unbounded();

    webserver(
        logger.clone(),
        build_output_root,
        network_descriptor,
        SocketAddr::new(config_bootstrap.ip.unwrap(), config_bootstrap.port.unwrap()),
        providers
            .iter()
            .map(|uri| Url::from_str(uri).unwrap())
            .collect(),
        &config_bootstrap.root.unwrap(),
        sender,
    )?
    .join()
    .map_err(|e| {
        DfxError::RuntimeError(Error::new(
            ErrorKind::Other,
            format!("Failed while running frontend proxy thead -- {:?}", e),
        ))
    })?;

    // Wait for the webserver to be started.
    let _ = receiver.recv().expect("Failed to receive server...");

    // Tell the user.
    info!(logger, "Webserver started...");

    // And then wait forever.
    loop {
        std::thread::sleep(Duration::from_secs(std::u64::MAX))
    }
}

/// Gets the configuration options for the bootstrap server. Each option is checked for correctness
/// and otherwise guaranteed to exist.
fn apply_arguments(
    config: &ConfigDefaultsBootstrap,
    env: &dyn Environment,
    args: &ArgMatches<'_>,
) -> DfxResult<ConfigDefaultsBootstrap> {
    let ip = get_ip(&config, args)?;
    let port = get_port(&config, args)?;
    let root = get_root(&config, env, args)?;
    let timeout = get_timeout(&config, args)?;
    Ok(ConfigDefaultsBootstrap {
        ip: Some(ip),
        port: Some(port),
        root: Some(root),
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
fn get_ip(config: &ConfigDefaultsBootstrap, args: &ArgMatches<'_>) -> DfxResult<IpAddr> {
    args.value_of("ip")
        .map(|ip| ip.parse())
        .unwrap_or_else(|| {
            let default = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
            Ok(config.ip.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid IP address: {}", err)))
}

/// Gets the port number that the bootstrap server listens on. First checks if the port number was
/// specified on the command-line using --port, otherwise checks if the port number was specified
/// in the dfx configuration file, otherise defaults to 8081.
fn get_port(config: &ConfigDefaultsBootstrap, args: &ArgMatches<'_>) -> DfxResult<u16> {
    args.value_of("port")
        .map(|port| port.parse())
        .unwrap_or_else(|| {
            let default = 8081;
            Ok(config.port.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid port number: {}", err)))
}

/// Gets the list of compute provider API endpoints.
fn get_providers(network_descriptor: &NetworkDescriptor) -> DfxResult<Vec<String>> {
    network_descriptor
        .providers
        .iter()
        .map(|url| Ok(format!("{}/api", url)))
        .collect()
}

/// Gets the directory containing static assets served by the bootstrap server. First checks if the
/// directory was specified on the command-line using --root, otherwise checks if the directory was
/// specified in the dfx configuration file, otherise defaults to
/// $HOME/.cache/dfinity/versions/$DFX_VERSION/bootstrap.
fn get_root(
    config: &ConfigDefaultsBootstrap,
    env: &dyn Environment,
    args: &ArgMatches<'_>,
) -> DfxResult<PathBuf> {
    args.value_of("root")
        .map(|root| parse_dir(root))
        .unwrap_or_else(|| {
            config
                .root
                .clone()
                .map_or(env.get_cache().get_binary_command_path("bootstrap"), Ok)
                .and_then(|root| {
                    parse_dir(
                        root.to_str()
                            .expect("File path without invalid unicode characters"),
                    )
                })
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid directory: {:?}", err)))
}

/// Gets the maximum amount of time, in seconds, the bootstrap server will wait for upstream
/// requests to complete. First checks if the timeout was specified on the command-line using
/// --timeout, otherwise checks if the timeout was specified in the dfx configuration file,
/// otherise defaults to 30.
fn get_timeout(config: &ConfigDefaultsBootstrap, args: &ArgMatches<'_>) -> DfxResult<u64> {
    args.value_of("timeout")
        .map(|timeout| timeout.parse())
        .unwrap_or_else(|| {
            let default = 30;
            Ok(config.timeout.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid timeout: {}", err)))
}

/// TODO (enzo): Documentation.
fn parse_dir(dir: &str) -> DfxResult<PathBuf> {
    fs::metadata(dir)
        .map(|_| PathBuf::from(dir))
        .map_err(|_| DfxError::Io(Error::new(ErrorKind::NotFound, dir)))
}
