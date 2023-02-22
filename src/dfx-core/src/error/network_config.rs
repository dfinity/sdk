use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkConfigError {
    #[error("Did not find any providers for network '{0}'")]
    NoProvidersForNetwork(String),

    #[error("Failed to parse URL '{0}': {1}")]
    ParseProviderUrlFailed(Box<String>, url::ParseError),
}
