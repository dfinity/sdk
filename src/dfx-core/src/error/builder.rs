use crate::error::identity::{InstantiateIdentityFromNameError, NewIdentityManagerError};
use crate::error::{
    load_dfx_config::LoadDfxConfigError, load_networks_config::LoadNetworksConfigError,
    network_config::NetworkConfigError, root_key::FetchRootKeyError,
};
use ic_agent::AgentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildDfxInterfaceError {
    #[error(transparent)]
    BuildAgent(#[from] BuildAgentError),

    #[error(transparent)]
    BuildIdentity(#[from] BuildIdentityError),

    #[error(transparent)]
    LoadNetworksConfig(#[from] LoadNetworksConfigError),

    #[error(transparent)]
    LoadDfxConfig(#[from] LoadDfxConfigError),

    #[error(transparent)]
    NetworkConfig(#[from] NetworkConfigError),

    #[error(transparent)]
    FetchRootKey(#[from] FetchRootKeyError),
}

#[derive(Error, Debug)]
pub enum BuildIdentityError {
    #[error(transparent)]
    NewIdentityManager(#[from] NewIdentityManagerError),

    #[error(transparent)]
    InstantiateIdentityFromName(#[from] InstantiateIdentityFromNameError),
}

#[derive(Error, Debug)]
pub enum BuildAgentError {
    #[error("failed to create http client")]
    CreateHttpClient(#[source] reqwest::Error),

    #[error("failed to create route provider")]
    CreateRouteProvider(#[source] AgentError),

    #[error("failed to create transportr")]
    CreateTransport(#[source] AgentError),

    #[error("failed to create agent")]
    CreateAgent(#[source] AgentError),
}
