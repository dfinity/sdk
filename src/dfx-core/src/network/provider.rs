use crate::config::directories::get_shared_network_data_directory;
use crate::config::model::dfinity::{
    Config, ConfigDefaults, ConfigLocalProvider, ConfigNetwork, NetworkType, NetworksConfig,
    DEFAULT_PROJECT_LOCAL_BIND, DEFAULT_SHARED_LOCAL_BIND,
};
use crate::config::model::local_server_descriptor::{
    LocalNetworkScopeDescriptor, LocalServerDescriptor,
};
use crate::config::model::network_descriptor::{
    NetworkDescriptor, NetworkTypeDescriptor, PLAYGROUND_NETWORK_NAME,
};
use crate::error::network_config::NetworkConfigError::{
    self, NetworkNotFound, NoNetworkContext, NoProvidersForNetwork, ParsePortValueFailed,
    ParseProviderUrlFailed, ReadWebserverPortFailed,
};
use crate::identity::WALLET_CONFIG_FILENAME;
use crate::util;
use lazy_static::lazy_static;
use serde_json::json;
use slog::{debug, info, warn, Logger};
use std::path::{Display, Path, PathBuf};
use std::sync::{Arc, RwLock};
use url::Url;

lazy_static! {
    static ref NETWORK_CONTEXT: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
}

fn set_network_context(network: Option<String>) {
    let name = network.unwrap_or_else(|| "local".to_string());

    let mut n = NETWORK_CONTEXT.write().unwrap();
    *n = Some(name);
}

pub fn get_network_context() -> Result<String, NetworkConfigError> {
    NETWORK_CONTEXT
        .read()
        .unwrap()
        .clone()
        .ok_or(NoNetworkContext())
}

pub enum LocalBindDetermination {
    /// Use value from configuration
    AsConfigured,

    /// Get port of running server from webserver-port file
    ApplyRunningWebserverPort,
}

fn config_network_to_network_descriptor(
    network_name: &str,
    config_network: &ConfigNetwork,
    project_defaults: Option<&ConfigDefaults>,
    data_directory: PathBuf,
    local_scope: LocalNetworkScopeDescriptor,
    ephemeral_wallet_config_path: &Path,
    local_bind_determination: &LocalBindDetermination,
    default_local_bind: &str,
    legacy_pid_path: Option<PathBuf>,
) -> Result<NetworkDescriptor, NetworkConfigError> {
    match config_network {
        ConfigNetwork::ConfigNetworkProvider(network_provider) => {
            let providers = if !network_provider.providers.is_empty() {
                network_provider
                    .providers
                    .iter()
                    .map(|provider| parse_provider_url(provider))
                    .collect::<Result<_, NetworkConfigError>>()
            } else {
                Err(NoProvidersForNetwork(network_name.to_string()))
            }?;
            let playground = network_provider.playground.clone();
            let is_ic = NetworkDescriptor::is_ic(network_name, &providers);
            Ok(NetworkDescriptor {
                name: network_name.to_string(),
                providers,
                r#type: NetworkTypeDescriptor::new(
                    network_provider.r#type,
                    ephemeral_wallet_config_path,
                    playground,
                )?,
                is_ic,
                local_server_descriptor: None,
            })
        }
        ConfigNetwork::ConfigLocalProvider(local_provider) => {
            let bitcoin = local_provider
                .bitcoin
                .clone()
                .or_else(|| project_defaults.and_then(|x| x.bitcoin.clone()))
                .unwrap_or_default();
            let canister_http = local_provider
                .canister_http
                .clone()
                .or_else(|| project_defaults.and_then(|x| x.canister_http.clone()))
                .unwrap_or_default();
            let replica = local_provider
                .replica
                .clone()
                .or_else(|| project_defaults.and_then(|x| x.replica.clone()))
                .unwrap_or_default();
            let playground = local_provider.playground.clone();

            let network_type = NetworkTypeDescriptor::new(
                local_provider.r#type,
                ephemeral_wallet_config_path,
                playground,
            )?;
            let bind_address = get_local_bind_address(
                local_provider,
                local_bind_determination,
                &data_directory,
                default_local_bind,
            )?;
            let provider_url = format!("http://{}", bind_address);
            let providers = vec![parse_provider_url(&provider_url)?];
            let local_server_descriptor = LocalServerDescriptor::new(
                data_directory,
                bind_address,
                bitcoin,
                canister_http,
                replica,
                local_scope,
                legacy_pid_path,
            )?;
            Ok(NetworkDescriptor {
                name: network_name.to_string(),
                providers,
                r#type: network_type,
                is_ic: false,
                local_server_descriptor: Some(local_server_descriptor),
            })
        }
    }
}

pub fn create_network_descriptor(
    project_config: Option<Arc<Config>>,
    shared_config: Arc<NetworksConfig>,
    network: Option<String>,
    logger: Option<Logger>,
    local_bind_determination: LocalBindDetermination,
) -> Result<NetworkDescriptor, NetworkConfigError> {
    let logger = (logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));

    set_network_context(network);
    let network_name = get_network_context()?;

    create_mainnet_network_descriptor(&network_name, &logger)
        .or_else(|| {
            create_project_network_descriptor(
                &network_name,
                project_config.clone(),
                &local_bind_determination,
                &logger,
            )
        })
        .or_else(|| {
            let project_config_for_warnings_only = project_config;
            create_shared_network_descriptor(
                &network_name,
                shared_config,
                project_config_for_warnings_only,
                &local_bind_determination,
                &logger,
            )
        })
        .or_else(|| create_default_network_from_name(&network_name, &logger).map(Ok))
        .or_else(|| create_url_based_network_descriptor(&network_name))
        .unwrap_or(Err(NetworkNotFound(network_name)))
}

fn create_mainnet_network_descriptor(
    network_name: &str,
    logger: &Logger,
) -> Option<Result<NetworkDescriptor, NetworkConfigError>> {
    if network_name == "ic" {
        info!(
            logger,
            "Using built-in definition for network 'ic' (mainnet)"
        );
        Some(Ok(NetworkDescriptor::ic()))
    } else {
        None
    }
}

fn create_url_based_network_descriptor(
    network_name: &str,
) -> Option<Result<NetworkDescriptor, NetworkConfigError>> {
    parse_provider_url(network_name).ok().map(|url| {
        // Replace any non-ascii-alphanumeric characters with `_`, to create an
        // OS-friendly directory name for it.
        let name = util::network_to_pathcompat(network_name);
        let is_ic = NetworkDescriptor::is_ic(&name, &vec![url.to_string()]);
        let data_directory = get_shared_network_data_directory(network_name)?;
        let network_type = NetworkTypeDescriptor::new(
            NetworkType::Ephemeral,
            &data_directory.join(WALLET_CONFIG_FILENAME),
            None,
        )?;
        Ok(NetworkDescriptor {
            name,
            providers: vec![url],
            r#type: network_type,
            is_ic,
            local_server_descriptor: None,
        })
    })
}

fn create_shared_network_descriptor(
    network_name: &str,
    shared_config: Arc<NetworksConfig>,
    project_config_for_warnings_only: Option<Arc<Config>>,
    local_bind_determination: &LocalBindDetermination,
    logger: &Logger,
) -> Option<Result<NetworkDescriptor, NetworkConfigError>> {
    let shared_config_file_exists = shared_config.get_path().is_file();
    let shared_config_display_path = shared_config.get_path().display();
    let network = shared_config.get_interface().get_network(network_name);
    let network = match (network_name, network) {
        ("local", None) => {
            if shared_config_file_exists {
                info!(logger, "Using the default definition for the 'local' shared network because {} does not define it.", shared_config_display_path);
            } else {
                info!(logger, "Using the default definition for the 'local' shared network because {} does not exist.", shared_config_display_path);
            }

            Some(ConfigNetwork::ConfigLocalProvider(ConfigLocalProvider {
                bind: Some(String::from(DEFAULT_SHARED_LOCAL_BIND)),
                r#type: NetworkType::Ephemeral,
                bitcoin: None,
                bootstrap: None,
                canister_http: None,
                replica: None,
                playground: None,
            }))
        }
        (network_name, None) => {
            if shared_config_file_exists {
                debug!(
                    logger,
                    "There is no shared network '{}' defined in {}",
                    &shared_config_display_path,
                    network_name
                );
            } else {
                debug!(
                    logger,
                    "There is no shared network '{}' because {} does not exist.",
                    network_name,
                    &shared_config_display_path
                );
            }
            None
        }
        (network_name, Some(network)) => {
            info!(
                logger,
                "Using shared network '{}' defined in {}",
                network_name,
                shared_config.get_path().display()
            );
            Some(network.clone())
        }
    };

    network.as_ref().map(|config_network| {
        warn_if_ignoring_project_defaults_in_shared_network(
            network_name,
            &shared_config_display_path,
            project_config_for_warnings_only,
            logger,
        );

        let data_directory = get_shared_network_data_directory(network_name)?;

        let ephemeral_wallet_config_path = data_directory.join(WALLET_CONFIG_FILENAME);

        let local_scope = LocalNetworkScopeDescriptor::shared(&data_directory);
        config_network_to_network_descriptor(
            network_name,
            config_network,
            None,
            data_directory,
            local_scope,
            &ephemeral_wallet_config_path,
            local_bind_determination,
            DEFAULT_SHARED_LOCAL_BIND,
            None,
        )
    })
}

fn warn_if_ignoring_project_defaults_in_shared_network(
    network_name: &str,
    shared_config_display_path: &Display,
    project_config_for_warnings_only: Option<Arc<Config>>,
    logger: &Logger,
) {
    if let Some(project_config_for_warnings_only) = project_config_for_warnings_only {
        let defaults = project_config_for_warnings_only.get_json().get("defaults");
        let bitcoin = defaults.and_then(|x| x.get("bitcoin")).cloned();
        let replica = defaults.and_then(|x| x.get("replica")).cloned();
        let canister_http = defaults.and_then(|x| x.get("canister_http")).cloned();

        if bitcoin.is_some() || replica.is_some() || canister_http.is_some() {
            let mut example_network_json = json!({});
            if let Some(bitcoin) = bitcoin {
                example_network_json["bitcoin"] = bitcoin;
            }
            if let Some(replica) = replica {
                example_network_json["replica"] = replica;
            }
            if let Some(canister_http) = canister_http {
                example_network_json["canister_http"] = canister_http;
            }
            let example_networks_json = json!({
                network_name: example_network_json
            });

            let example_networks_json =
                serde_json::to_string_pretty(&example_networks_json).unwrap();
            warn!(logger, "Ignoring the 'defaults' field in dfx.json because project settings never apply to the shared network.\n\
                    To apply these settings to the shared network, define them in {shared_config_display_path} like so:\n\
                    {example_networks_json}",
                    shared_config_display_path = &shared_config_display_path,
                    example_networks_json = &example_networks_json);
        }
    }
}

fn create_project_network_descriptor(
    network_name: &str,
    project_config: Option<Arc<Config>>,
    local_bind_determination: &LocalBindDetermination,
    logger: &Logger,
) -> Option<Result<NetworkDescriptor, NetworkConfigError>> {
    if let Some(config) = project_config {
        if let Some(config_network) = config.get_config().get_network(network_name) {
            info!(
                logger,
                "Using project-specific network '{}' defined in {}",
                network_name,
                config.get_path().display(),
            );

            let data_directory = config.get_temp_path().join("network").join(network_name);
            let legacy_pid_path = Some(config.get_temp_path().join("pid"));
            let ephemeral_wallet_config_path = config
                .get_temp_path()
                .join("local")
                .join(WALLET_CONFIG_FILENAME);
            Some(config_network_to_network_descriptor(
                network_name,
                config_network,
                Some(config.get_config().get_defaults()),
                data_directory,
                LocalNetworkScopeDescriptor::Project,
                &ephemeral_wallet_config_path,
                local_bind_determination,
                DEFAULT_PROJECT_LOCAL_BIND,
                legacy_pid_path,
            ))
        } else {
            debug!(
                logger,
                "There is no project-specific network '{}' defined in {}",
                network_name,
                config.get_path().display()
            );
            None
        }
    } else {
        debug!(
            logger,
            "There is no project-specific network '{}' because there is no project (no dfx.json).",
            network_name
        );
        None
    }
}

fn create_default_network_from_name(
    network_name: &str,
    logger: &Logger,
) -> Option<NetworkDescriptor> {
    match network_name {
        PLAYGROUND_NETWORK_NAME => {
            debug!(logger, "Using default definition for network 'playground'.");
            Some(NetworkDescriptor::default_playground_network())
        }
        _ => None,
    }
}

fn get_local_bind_address(
    local_provider: &ConfigLocalProvider,
    local_bind_determination: &LocalBindDetermination,
    data_directory: &Path,
    default_local_bind: &str,
) -> Result<String, NetworkConfigError> {
    match local_bind_determination {
        LocalBindDetermination::AsConfigured => Ok(local_provider
            .bind
            .clone()
            .unwrap_or_else(|| default_local_bind.to_string())),
        LocalBindDetermination::ApplyRunningWebserverPort => {
            get_running_webserver_bind_address(data_directory, local_provider, default_local_bind)
        }
    }
}

fn get_running_webserver_bind_address(
    data_directory: &Path,
    local_provider: &ConfigLocalProvider,
    default_local_bind: &str,
) -> Result<String, NetworkConfigError> {
    let local_bind = local_provider
        .bind
        .clone()
        .unwrap_or_else(|| default_local_bind.to_string());
    let path = data_directory.join("webserver-port");
    if path.exists() {
        let s = crate::fs::read_to_string(&path).map_err(ReadWebserverPortFailed)?;
        let s = s.trim();
        if s.is_empty() {
            Ok(local_bind)
        } else {
            let port = s
                .parse::<u16>()
                .map_err(|e| ParsePortValueFailed(Box::new(path), Box::new(e)))?;
            // converting to a socket address, and then setting the port,
            // will unfortunately transform "localhost:port" to "[::1]:{port}",
            // which the agent fails to connect with.
            let host = match local_bind.rfind(':') {
                None => local_bind.clone(),
                Some(index) => local_bind[0..index].to_string(),
            };
            Ok(format!("{}:{}", host, port))
        }
    } else {
        Ok(local_bind)
    }
}

pub fn command_line_provider_to_url(s: &str) -> Result<String, NetworkConfigError> {
    match parse_provider_url(s) {
        Ok(url) => Ok(url),
        Err(original_error) => {
            let prefixed_with_http = format!("http://{}", s);
            parse_provider_url(&prefixed_with_http).or(Err(original_error))
        }
    }
}

pub fn parse_provider_url(url: &str) -> Result<String, NetworkConfigError> {
    Url::parse(url)
        .map(|_| String::from(url))
        .map_err(|e| ParseProviderUrlFailed(Box::new(url.to_string()), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::bitcoin_adapter::BitcoinAdapterLogLevel;
    use crate::config::model::canister_http_adapter::HttpAdapterLogLevel;
    use crate::config::model::dfinity::ReplicaSubnetType::{System, VerifiedApplication};
    use crate::config::model::dfinity::{
        to_socket_addr, ConfigDefaultsBitcoin, ConfigDefaultsCanisterHttp, ConfigDefaultsReplica,
        ReplicaLogLevel,
    };
    use std::fs;
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[test]
    fn use_default_if_no_webserver_port_file() {
        // no file - use default
        test_with_webserver_port_file_contents(
            LocalBindDetermination::ApplyRunningWebserverPort,
            None,
            "localhost:8000",
        );
    }

    #[test]
    fn ignore_running_webserver_port_if_not_requested() {
        // port file present and populated, but not asked for: ignored
        test_with_webserver_port_file_contents(
            LocalBindDetermination::AsConfigured,
            Some("1234"),
            "localhost:8000",
        );
    }

    #[test]
    fn use_port_if_have_file() {
        // port file present and populated: reflected in socket address
        test_with_webserver_port_file_contents(
            LocalBindDetermination::ApplyRunningWebserverPort,
            Some("1234"),
            "localhost:1234",
        );
    }

    #[test]
    fn ignore_port_if_not_requested() {
        // port file present and populated, but not asked for: ignored
        test_with_webserver_port_file_contents(
            LocalBindDetermination::AsConfigured,
            Some("1234"),
            "localhost:8000",
        );
    }

    #[test]
    fn extra_whitespace_in_webserver_port_is_ok() {
        // trailing newline is ok
        test_with_webserver_port_file_contents(
            LocalBindDetermination::ApplyRunningWebserverPort,
            Some("  \n3456 \n"),
            "localhost:3456",
        );
    }

    #[test]
    fn use_running_webserver_address() {
        // no file - use default
        test_with_webserver_port_file_contents(
            LocalBindDetermination::ApplyRunningWebserverPort,
            None,
            "localhost:8000",
        );
    }

    #[test]
    fn ignore_empty_webserver_port_file() {
        // empty is ok: ignore
        test_with_webserver_port_file_contents(
            LocalBindDetermination::ApplyRunningWebserverPort,
            Some(""),
            "localhost:8000",
        );
    }
    #[test]
    fn ignore_whitespace_only_webserver_port_file() {
        // just whitespace is ok: ignore
        test_with_webserver_port_file_contents(
            LocalBindDetermination::ApplyRunningWebserverPort,
            Some("\n"),
            "localhost:8000",
        );
    }

    fn test_with_webserver_port_file_contents(
        local_bind_determination: LocalBindDetermination,
        webserver_port_contents: Option<&str>,
        expected_socket_addr: &str,
    ) {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let project_dfx_json = project_dir.join("dfx.json");
        std::fs::write(
            project_dfx_json,
            r#"{
            "networks": {
                "local": {
                    "bind": "localhost:8000"
                }
            }
        }"#,
        )
        .unwrap();

        if let Some(webserver_port_contents) = webserver_port_contents {
            let dot_dfx_dir = project_dir.join(".dfx");
            let network_data_dir = dot_dfx_dir.join("network").join("local");
            fs::create_dir_all(&network_data_dir).unwrap();
            std::fs::write(
                network_data_dir.join("webserver-port"),
                webserver_port_contents,
            )
            .unwrap();
        }

        let mut no_op_transformer =
            crate::extension::manifest::custom_canister_type::NoopTransformConfiguration;
        let config = Config::from_dir(&project_dir, &mut no_op_transformer)
            .unwrap()
            .unwrap();
        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            local_bind_determination,
        )
        .unwrap();

        assert_eq!(
            network_descriptor
                .local_server_descriptor()
                .unwrap()
                .bind_address,
            to_socket_addr(expected_socket_addr).unwrap()
        );
    }

    #[test]
    fn config_with_local_bind_addr() {
        let config = Config::from_str(
            r#"{
            "networks": {
                "local": {
                    "bind": "localhost:8000"
                }
            }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();

        assert_eq!(
            network_descriptor
                .local_server_descriptor()
                .unwrap()
                .bind_address,
            to_socket_addr("localhost:8000").unwrap()
        );
    }

    #[test]
    fn config_with_invalid_local_bind_addr() {
        let config = Config::from_str(
            r#"{
            "networks": {
                "local": {
                    "bind": "not a valid bind address"
                }
            }
        }"#,
        )
        .unwrap();

        let result = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        );
        assert!(result.is_err());
    }

    #[test]
    fn config_returns_local_bind_address_if_no_local_network() {
        let config = Config::from_str(
            r#"{
            "networks": {
            }
        }"#,
        )
        .unwrap();
        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();

        assert_eq!(
            network_descriptor
                .local_server_descriptor()
                .unwrap()
                .bind_address,
            to_socket_addr("127.0.0.1:4943").unwrap()
        );
    }

    #[test]
    fn config_returns_local_bind_address_if_no_networks() {
        let config = Config::from_str(
            r#"{
        }"#,
        )
        .unwrap();
        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();

        assert_eq!(
            network_descriptor
                .local_server_descriptor()
                .unwrap()
                .bind_address,
            to_socket_addr("127.0.0.1:4943").unwrap()
        );
    }

    #[test]
    fn get_bitcoin_config() {
        let config = Config::from_str(
            r#"{
              "defaults": {
                "bitcoin": {
                  "enabled": true,
                  "nodes": ["127.0.0.1:18444"],
                  "log_level": "info"
                }
              },
              "networks": {
                "local": {
                    "bind": "localhost:8000"
                }
              }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();
        let bitcoin_config = &network_descriptor
            .local_server_descriptor()
            .unwrap()
            .bitcoin;

        assert_eq!(
            bitcoin_config,
            &ConfigDefaultsBitcoin {
                enabled: true,
                nodes: Some(vec![SocketAddr::from_str("127.0.0.1:18444").unwrap()]),
                log_level: BitcoinAdapterLogLevel::Info,
                ..ConfigDefaultsBitcoin::default()
            }
        );
    }

    #[test]
    fn get_bitcoin_config_default_log_level() {
        let config = Config::from_str(
            r#"{
              "defaults": {
                "bitcoin": {
                  "enabled": true,
                  "nodes": ["127.0.0.1:18444"]
                }
              },
              "networks": {
                "local": {
                    "bind": "localhost:8000"
                }
              }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();
        let bitcoin_config = &network_descriptor
            .local_server_descriptor()
            .unwrap()
            .bitcoin;

        assert_eq!(
            bitcoin_config,
            &ConfigDefaultsBitcoin {
                enabled: true,
                nodes: Some(vec![SocketAddr::from_str("127.0.0.1:18444").unwrap()]),
                log_level: BitcoinAdapterLogLevel::Info, // A default log level of "info" is assumed
                ..ConfigDefaultsBitcoin::default()
            }
        );
    }

    #[test]
    fn get_bitcoin_config_debug_log_level() {
        let config = Config::from_str(
            r#"{
              "defaults": {
                "bitcoin": {
                  "enabled": true,
                  "log_level": "debug"
                }
              },
              "networks": {
                "local": {
                    "bind": "localhost:8000"
                }
              }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();
        let bitcoin_config = &network_descriptor
            .local_server_descriptor()
            .unwrap()
            .bitcoin;

        assert_eq!(
            bitcoin_config,
            &ConfigDefaultsBitcoin {
                enabled: true,
                log_level: BitcoinAdapterLogLevel::Debug,
                ..ConfigDefaultsBitcoin::default()
            }
        );
    }

    #[test]
    fn bitcoin_config_on_local_network() {
        let config = Config::from_str(
            r#"{
              "networks": {
                "local": {
                  "bind": "127.0.0.1:8000",
                  "bitcoin": {
                    "enabled": true,
                    "nodes": ["127.0.0.1:18444"],
                    "log_level": "info"
                  }
                }
              }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();
        let bitcoin_config = &network_descriptor
            .local_server_descriptor()
            .unwrap()
            .bitcoin;

        assert_eq!(
            bitcoin_config,
            &ConfigDefaultsBitcoin {
                enabled: true,
                nodes: Some(vec![SocketAddr::from_str("127.0.0.1:18444").unwrap()]),
                log_level: BitcoinAdapterLogLevel::Info,
                ..ConfigDefaultsBitcoin::default()
            }
        );
    }

    #[test]
    fn replica_config_on_local_network() {
        let config = Config::from_str(
            r#"{
              "networks": {
                "local": {
                  "bind": "127.0.0.1:8000",
                  "replica": {
                    "subnet_type": "verifiedapplication",
                    "port": 17001,
                    "log_level": "trace"
                  }
                }
              }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();
        let replica_config = &network_descriptor
            .local_server_descriptor()
            .unwrap()
            .replica;

        assert_eq!(
            replica_config,
            &ConfigDefaultsReplica {
                subnet_type: Some(VerifiedApplication),
                port: Some(17001),
                log_level: Some(ReplicaLogLevel::Trace)
            }
        );
    }

    #[test]
    fn replica_config_on_local_network_overrides_default() {
        // Defaults are not combined.
        // Here the 'default' level specifies a port, but it's ignored due to the
        // network-level setting.
        let config = Config::from_str(
            r#"{
              "defaults": {
                "replica": {
                  "port": 13131
                }
              },
              "networks": {
                "local": {
                  "bind": "127.0.0.1:8000",
                  "replica": {
                    "subnet_type": "system"
                  }
                }
              }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();
        let replica_config = &network_descriptor
            .local_server_descriptor()
            .unwrap()
            .replica;

        assert_eq!(
            replica_config,
            &ConfigDefaultsReplica {
                subnet_type: Some(System),
                port: None,
                log_level: None
            }
        );
    }

    #[test]
    fn canister_http_config_on_local_network() {
        let config = Config::from_str(
            r#"{
              "networks": {
                "local": {
                  "bind": "127.0.0.1:8000",
                  "canister_http": {
                    "enabled": true,
                    "log_level": "debug"
                  }
                }
              }
        }"#,
        )
        .unwrap();

        let network_descriptor = create_network_descriptor(
            Some(Arc::new(config)),
            Arc::new(NetworksConfig::new().unwrap()),
            None,
            None,
            LocalBindDetermination::AsConfigured,
        )
        .unwrap();
        let canister_http_config = &network_descriptor
            .local_server_descriptor()
            .unwrap()
            .canister_http;

        assert_eq!(
            canister_http_config,
            &ConfigDefaultsCanisterHttp {
                enabled: true,
                log_level: HttpAdapterLogLevel::Debug
            }
        );
    }

    #[test]
    fn url_is_url() {
        assert_eq!(
            command_line_provider_to_url("http://127.0.0.1:8000").unwrap(),
            "http://127.0.0.1:8000"
        );
    }

    #[test]
    fn addr_and_port_to_url() {
        assert_eq!(
            command_line_provider_to_url("127.0.0.1:8000").unwrap(),
            "http://127.0.0.1:8000"
        );
    }
}
