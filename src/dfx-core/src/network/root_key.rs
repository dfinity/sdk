use crate::{
    config::model::network_descriptor::NetworkDescriptor, error::root_key::FetchRootKeyError,
};
use ic_agent::Agent;

pub async fn fetch_root_key_when_local(
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

pub async fn fetch_root_key_when_local_or_error(
    agent: &Agent,
    network: &NetworkDescriptor,
) -> Result<(), FetchRootKeyError> {
    if !network.is_ic {
        agent
            .fetch_root_key()
            .await
            .map_err(FetchRootKeyError::AgentError)
    } else {
        Err(FetchRootKeyError::NetworkMustBeLocal)
    }
}
