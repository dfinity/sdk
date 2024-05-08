use crate::config::model::dfinity::{Config, NetworksConfig};
use crate::config::model::network_descriptor::NetworkDescriptor;
use crate::error::builder::BuildDfxInterfaceError;
use crate::DfxInterfaceBuilder;
use ic_agent::Agent;
use std::sync::Arc;

pub struct DfxInterface {
    pub(crate) config: Option<Arc<Config>>,
    pub(crate) agent: Agent,
    pub(crate) networks_config: NetworksConfig,
    pub(crate) network_descriptor: NetworkDescriptor,
}

impl DfxInterface {
    pub fn builder() -> DfxInterfaceBuilder {
        DfxInterfaceBuilder::new()
    }

    pub async fn anonymous() -> Result<DfxInterface, BuildDfxInterfaceError> {
        DfxInterfaceBuilder::new().anonymous().build().await
    }

    pub fn config(&self) -> Option<Arc<Config>> {
        self.config.clone()
    }

    pub fn agent(&self) -> &Agent {
        &self.agent
    }

    pub fn networks_config(&self) -> &NetworksConfig {
        &self.networks_config
    }

    pub fn network_descriptor(&self) -> &NetworkDescriptor {
        &self.network_descriptor
    }
}
