use crate::{
    config::model::network_descriptor::NetworkDescriptor, error::root_key::FetchRootKeyError,
};
use ic_agent::Agent;

#[deprecated(note = "use fetch_root_key_when_non_mainnet() instead")]
pub async fn fetch_root_key_when_local(
    agent: &Agent,
    network: &NetworkDescriptor,
) -> Result<(), FetchRootKeyError> {
    fetch_root_key_when_non_mainnet(agent, network).await
}

pub async fn fetch_root_key_when_non_mainnet(
    agent: &Agent,
    network: &NetworkDescriptor,
) -> Result<(), FetchRootKeyError> {
    if !network.is_ic {
        agent
            .fetch_root_key()
            .await
            .map_err(FetchRootKeyError::AgentError)?;
    }
    Ok(())
}

#[deprecated(note = "use fetch_root_key_when_non_mainnet_or_error() instead")]
pub async fn fetch_root_key_when_local_or_error(
    agent: &Agent,
    network: &NetworkDescriptor,
) -> Result<(), FetchRootKeyError> {
    fetch_root_key_when_non_mainnet_or_error(agent, network).await
}

pub async fn fetch_root_key_when_non_mainnet_or_error(
    agent: &Agent,
    network: &NetworkDescriptor,
) -> Result<(), FetchRootKeyError> {
    if !network.is_ic {
        agent
            .fetch_root_key()
            .await
            .map_err(FetchRootKeyError::AgentError)
    } else {
        Err(FetchRootKeyError::MustNotFetchRootKeyOnMainnet)
    }
}
