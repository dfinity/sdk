use crate::config::dfinity::ConfigNetwork;
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use clap::ArgMatches;
use url::Url;

// always returns at least one url
pub fn get_provider_urls<'a>(
    env: &'a (dyn Environment + 'a),
    args: &ArgMatches<'_>,
) -> DfxResult<Vec<String>> {
    args.value_of("provider")
        .map_or_else::<DfxResult<Vec<String>>, _, _>(
            || {
                let network_name = args.value_of("network").unwrap_or("local");
                let config = env
                    .get_config()
                    .ok_or(DfxError::CommandMustBeRunInAProject)?;
                let config = config.as_ref().get_config();
                match config.get_network(&network_name) {
                    Some(ConfigNetwork::ConfigNetworkProvider(network_provider)) => {
                        match &network_provider.providers {
                            providers if !providers.is_empty() => Ok(providers.to_vec()),
                            _ => Err(DfxError::ComputeNetworkHasNoProviders(
                                network_name.to_string(),
                            )),
                        }
                    }
                    Some(ConfigNetwork::ConfigLocalProvider(local_provider)) => {
                        Ok(vec![format!("http://{}", local_provider.bind)])
                    }
                    None => Err(DfxError::ComputeNetworkNotFound(network_name.to_string())),
                }?
                .iter()
                .map(|provider| parse_provider_url(provider))
                .collect::<Result<_, _>>()
            },
            |provider| command_line_provider_to_url(&provider).map(|url| vec![url]),
        )
}

pub fn get_first_agent_url<'a>(
    env: &'a (dyn Environment + 'a),
    args: &ArgMatches<'_>,
) -> DfxResult<String> {
    let urls = get_provider_urls(env, args);
    urls.map(|urls| urls.first().unwrap().to_string())
}

pub fn create_agent_environment<'a>(
    env: &'a (dyn Environment + 'a),
    args: &ArgMatches<'_>,
) -> DfxResult<AgentEnvironment<'a>> {
    let agent_url = get_first_agent_url(env, args)?;

    Ok(AgentEnvironment::new(env, &agent_url))
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
