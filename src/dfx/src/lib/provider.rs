use crate::config::dfinity::ConfigNetwork;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::network::network_descriptor::NetworkDescriptor;
use clap::ArgMatches;
use url::Url;

// always returns at least one url
pub fn get_network_descriptor<'a>(
    env: &'a (dyn Environment + 'a),
    args: &ArgMatches<'_>,
) -> DfxResult<NetworkDescriptor> {
    let network_name = args.value_of("network").unwrap_or("local");
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let config = config.as_ref().get_config();
    match config.get_network(&network_name) {
        Some(ConfigNetwork::ConfigNetworkProvider(network_provider)) => {
            let provider_urls = match &network_provider.providers {
                providers if !providers.is_empty() => {
                    let provider_urls = providers.to_vec();

                    Ok(provider_urls)
                }
                _ => Err(DfxError::ComputeNetworkHasNoProviders(
                    network_name.to_string(),
                )),
            }?;
            let validated_urls = provider_urls
                .iter()
                .map(|provider| parse_provider_url(provider))
                .collect::<Result<_, _>>();
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
                .collect::<Result<_, _>>();
            validated_urls.map(|provider_urls| NetworkDescriptor {
                name: network_name.to_string(),
                providers: provider_urls,
                r#type: local_provider.r#type,
            })
        }
        None => Err(DfxError::ComputeNetworkNotFound(network_name.to_string())),
    }
}

pub fn create_agent_environment<'a>(
    env: &'a (dyn Environment + 'a),
    args: &ArgMatches<'_>,
) -> DfxResult<AgentEnvironment<'a>> {
    let network_descriptor = get_network_descriptor(env, args)?;

    Ok(AgentEnvironment::new(env, network_descriptor))
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
        .map_err(|err| DfxError::InvalidUrl(url.to_string(), err))
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
