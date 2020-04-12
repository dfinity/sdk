use crate::config::dfinity::ConfigDefaultsBootstrap;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::webserver::webserver;
use clap::{App, Arg, ArgMatches};
use slog::info;
use std::default::Default;
use std::fs;
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use url::{ParseError, Url};

/// Constructs a sub-command to run the bootstrap server.
pub fn construct() -> App<'static> {
    App::new("bootstrap")
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
            Arg::with_name("providers")
                .help(UserMessage::BootstrapProviders.to_str())
                .long("providers")
                .multiple(true)
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
pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let logger = env.get_logger();
    let config = get_config(env, args)?;

    let (sender, receiver) = crossbeam::unbounded();

    webserver(
        logger.clone(),
        SocketAddr::new(config.ip.unwrap(), config.port.unwrap()),
        config
            .providers
            .map(|vec| vec.iter().map(|uri| Url::from_str(uri).unwrap()).collect())
            .unwrap(),
        &config.root.unwrap(),
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
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Gets the configuration options for the bootstrap server. Each option is checked for correctness
/// and otherwise guaranteed to exist.
fn get_config(env: &dyn Environment, args: &ArgMatches) -> DfxResult<ConfigDefaultsBootstrap> {
    let config = get_config_from_file(env);
    let ip = get_ip(&config, args)?;
    let port = get_port(&config, args)?;
    let providers = get_providers(&config, args)?;
    let root = get_root(&config, env, args)?;
    let timeout = get_timeout(&config, args)?;
    Ok(ConfigDefaultsBootstrap {
        ip: Some(ip),
        port: Some(port),
        providers: Some(providers),
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
fn get_ip(config: &ConfigDefaultsBootstrap, args: &ArgMatches) -> DfxResult<IpAddr> {
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
fn get_port(config: &ConfigDefaultsBootstrap, args: &ArgMatches) -> DfxResult<u16> {
    args.value_of("port")
        .map(|port| port.parse())
        .unwrap_or_else(|| {
            let default = 8081;
            Ok(config.port.unwrap_or(default))
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid port number: {}", err)))
}

/// Gets the list of compute provider API endpoints. First checks if the providers were specified
/// on the command-line using --providers, otherwise checks if the providers were specified in the
/// dfx configuration file, otherwise defaults to http://127.0.0.1:8080/api.
fn get_providers(
    config: &ConfigDefaultsBootstrap,
    args: &ArgMatches,
) -> DfxResult<Vec<String>> {
    args.values_of("providers")
        .map(|providers| {
            providers
                .map(|provider| parse_url(provider))
                .collect::<Result<_, _>>()
        })
        .unwrap_or_else(|| {
            let default = vec!["http://127.0.0.1:8080/api".to_string()];
            config.providers.clone().map_or(Ok(default), |providers| {
                if providers.is_empty() {
                    Err(ParseError::EmptyHost)
                } else {
                    providers
                        .iter()
                        .map(|provider| parse_url(provider))
                        .collect()
                }
            })
        })
        .map_err(|err| DfxError::InvalidArgument(format!("Invalid provider: {}", err)))
}

/// Gets the directory containing static assets served by the bootstrap server. First checks if the
/// directory was specified on the command-line using --root, otherwise checks if the directory was
/// specified in the dfx configuration file, otherise defaults to
/// $HOME/.cache/dfinity/versions/$DFX_VERSION/js-user-library/dist/bootstrap.
fn get_root(
    config: &ConfigDefaultsBootstrap,
    env: &dyn Environment,
    args: &ArgMatches,
) -> DfxResult<PathBuf> {
    args.value_of("root")
        .map(|root| parse_dir(root))
        .unwrap_or_else(|| {
            config
                .root
                .clone()
                .map_or(
                    env.get_cache()
                        .get_binary_command_path("js-user-library/dist/bootstrap"),
                    Ok,
                )
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
fn get_timeout(config: &ConfigDefaultsBootstrap, args: &ArgMatches) -> DfxResult<u64> {
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

/// TODO (enzo): Documentation.
fn parse_url(url: &str) -> Result<String, ParseError> {
    Url::parse(url).map(|_| String::from(url))
}
