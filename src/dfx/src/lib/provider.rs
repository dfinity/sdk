use crate::config::dfinity::{Config, ConfigNetwork, NetworkType};
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::DfxResult;
use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util::{self, expiry_duration};

use anyhow::{anyhow, Context};
use fn_error_context::context;
use lazy_static::lazy_static;
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

#[context("Failed to get network context.")]
pub fn get_network_context() -> DfxResult<String> {
    NETWORK_CONTEXT
        .read()
        .unwrap()
        .clone()
        .ok_or_else(|| anyhow!("Cannot find network context."))
}

// always returns at least one url
#[context("Failed to get network descriptor.")]
pub fn get_network_descriptor(
    config: Option<Arc<Config>>,
    network: Option<String>,
) -> DfxResult<NetworkDescriptor> {
    set_network_context(network);
    let config = config.unwrap_or_else(|| {
        eprintln!("dfx.json not found, using default.");
        Arc::new(Config::from_str("{}").unwrap())
    });
    let config = config.as_ref().get_config();
    let network_name = get_network_context()?;
    match config.get_network(&network_name) {
        Some(ConfigNetwork::ConfigNetworkProvider(network_provider)) => {
            let provider_urls = match &network_provider.providers {
                providers if !providers.is_empty() => Ok(providers.to_vec()),
                _ => Err(anyhow!(
                    "Cannot find providers for network \"{}\"",
                    network_name
                )),
            }?;
            let validated_urls = provider_urls
                .iter()
                .map(|provider| parse_provider_url(provider))
                .collect::<DfxResult<_>>();
            validated_urls.map(|provider_urls| NetworkDescriptor {
                name: network_name.to_string(),
                r#type: network_provider.r#type,
                is_ic: NetworkDescriptor::is_ic(&network_name.to_string(), &provider_urls),
                providers: provider_urls,
                local_server_descriptor: None,
            })
        }
        Some(ConfigNetwork::ConfigLocalProvider(local_provider)) => {
            let network_type = local_provider.r#type;
            let bind_address = local_provider.bind;
            let provider_urls = vec![format!("http://{}", bind_address)];
            let validated_urls = provider_urls
                .iter()
                .map(|provider| parse_provider_url(provider))
                .collect::<DfxResult<_>>();
            validated_urls.map(|provider_urls| NetworkDescriptor {
                name: network_name.to_string(),
                providers: provider_urls,
                r#type: network_type,
                is_ic: false,
                local_server_descriptor: Some(LocalServerDescriptor { bind: bind_address }),
            })
        }
        None => {
            // Allow a URL to be specified as a network (if it's parseable as a URL).
            if let Ok(url) = parse_provider_url(&network_name) {
                // Replace any non-ascii-alphanumeric characters with `_`, to create an
                // OS-friendly directory name for it.
                let name = util::network_to_pathcompat(&network_name);
                let is_ic = NetworkDescriptor::is_ic(&name, &vec![url.to_string()]);

                Ok(NetworkDescriptor {
                    name,
                    providers: vec![url],
                    r#type: NetworkType::Ephemeral,
                    is_ic,
                    local_server_descriptor: None,
                })
            } else {
                Err(anyhow!("ComputeNetworkNotFound({})", network_name))
            }
        }
    }
}

#[context("Failed to create AgentEnvironment.")]
pub fn create_agent_environment<'a>(
    env: &'a (dyn Environment + 'a),
    network: Option<String>,
) -> DfxResult<AgentEnvironment<'a>> {
    let network_descriptor = get_network_descriptor(env.get_config(), network)?;
    let timeout = expiry_duration();
    AgentEnvironment::new(env, network_descriptor, timeout)
}

#[context("Failed to parse supplied provider url {}.", s)]
pub fn command_line_provider_to_url(s: &str) -> DfxResult<String> {
    match parse_provider_url(s) {
        Ok(url) => Ok(url),
        Err(original_error) => {
            let prefixed_with_http = format!("http://{}", s);
            parse_provider_url(&prefixed_with_http).or(Err(original_error))
        }
    }
}

pub fn parse_provider_url(url: &str) -> DfxResult<String> {
    Url::parse(url)
        .map(|_| String::from(url))
        .with_context(|| format!("Cannot parse provider URL {}.", url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::dfinity::to_socket_addr;

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

        let network_descriptor = get_network_descriptor(Some(Arc::new(config)), None).unwrap();

        assert_eq!(
            network_descriptor
                .local_server_descriptor()
                .unwrap()
                .bind_address()
                .unwrap(),
            to_socket_addr("localhost:8000").unwrap()
        );
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
        let network_descriptor = get_network_descriptor(Some(Arc::new(config)), None).unwrap();

        assert_eq!(
            network_descriptor
                .local_server_descriptor()
                .unwrap()
                .bind_address()
                .unwrap(),
            to_socket_addr("127.0.0.1:8000").unwrap()
        );
    }

    #[test]
    fn config_returns_local_bind_address_if_no_networks() {
        let config = Config::from_str(
            r#"{
        }"#,
        )
        .unwrap();
        let network_descriptor = get_network_descriptor(Some(Arc::new(config)), None).unwrap();

        assert_eq!(
            network_descriptor
                .local_server_descriptor()
                .unwrap()
                .bind_address()
                .unwrap(),
            to_socket_addr("127.0.0.1:8000").unwrap()
        );
    }

    #[test]
    fn url_is_url() {
        assert_eq!(
            command_line_provider_to_url(&"http://127.0.0.1:8000".to_string()).unwrap(),
            "http://127.0.0.1:8000"
        );
    }

    #[test]
    fn addr_and_port_to_url() {
        assert_eq!(
            command_line_provider_to_url(&"127.0.0.1:8000".to_string()).unwrap(),
            "http://127.0.0.1:8000"
        );
    }
}
