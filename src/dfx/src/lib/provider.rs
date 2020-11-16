use crate::config::dfinity::ConfigNetwork;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::DfxResult;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::util::expiry_duration;

use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use std::io::{Error, ErrorKind};
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

pub fn get_network_context() -> DfxResult<String> {
    NETWORK_CONTEXT
        .read()
        .unwrap()
        .clone()
        .ok_or_else(|| anyhow!("Cannot find network context."))
}

// always returns at least one url
pub fn get_network_descriptor<'a>(
    env: &'a (dyn Environment + 'a),
    network: Option<String>,
) -> DfxResult<NetworkDescriptor> {
    set_network_context(network);
    let config = env
        .get_config()
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "dfx.json"))?;
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
                providers: provider_urls,
                r#type: network_provider.r#type,
            })
        }
        Some(ConfigNetwork::ConfigLocalProvider(local_provider)) => {
            let provider_urls = vec![format!("http://{}", local_provider.bind)];
            let validated_urls = provider_urls
                .iter()
                .map(|provider| parse_provider_url(provider))
                .collect::<DfxResult<_>>();
            validated_urls.map(|provider_urls| NetworkDescriptor {
                name: network_name.to_string(),
                providers: provider_urls,
                r#type: local_provider.r#type,
            })
        }
        None => Err(anyhow!("Cannot find network \"{}\"", network_name)),
    }
}

pub fn create_agent_environment<'a>(
    env: &'a (dyn Environment + 'a),
    network: Option<String>,
) -> DfxResult<AgentEnvironment<'a>> {
    let network_descriptor = get_network_descriptor(env, network)?;
    let timeout = expiry_duration();
    AgentEnvironment::new(env, network_descriptor, timeout)
}

pub fn command_line_provider_to_url(s: &str) -> DfxResult<String> {
    match parse_provider_url(&s) {
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
        .context("Cannot parse provider URL.")
}

#[cfg(test)]
mod tests {
    use super::*;

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
