use crate::{
    config::model::{
        dfinity::{Config, NetworksConfig},
        network_descriptor::NetworkDescriptor,
    },
    error::{
        builder::{BuildAgentError, BuildDfxInterfaceError, BuildIdentityError},
        network_config::NetworkConfigError,
    },
    identity::{identity_manager::InitializeIdentity, IdentityManager},
    network::{
        provider::{create_network_descriptor, LocalBindDetermination},
        root_key::fetch_root_key_when_non_mainnet_or_error,
    },
    DfxInterface,
};
use ic_agent::{
    agent::http_transport::{
        reqwest_transport::ReqwestHttpReplicaV2Transport, route_provider::RoundRobinRouteProvider,
    },
    Agent, Identity,
};
use reqwest::Client;
use std::sync::Arc;

#[derive(PartialEq)]
pub enum IdentityPicker {
    Anonymous,
    Selected,
    Named(String),
}

#[derive(PartialEq)]
pub enum NetworkPicker {
    Local,
    Mainnet,
    Named(String),
}

pub struct DfxInterfaceBuilder {
    identity: IdentityPicker,

    network: NetworkPicker,

    /// Force fetching of the root key.
    /// This is insecure and should only be set for non-mainnet networks.
    /// There is no need to set this for the local network, where the root key is fetched by default.
    /// This would typically be set for a testnet, or an alias for the local network.
    force_fetch_root_key_insecure_non_mainnet_only: bool,
}

impl DfxInterfaceBuilder {
    pub(crate) fn new() -> Self {
        Self {
            identity: IdentityPicker::Selected,
            network: NetworkPicker::Local,
            force_fetch_root_key_insecure_non_mainnet_only: false,
        }
    }

    pub fn anonymous(self) -> Self {
        self.with_identity(IdentityPicker::Anonymous)
    }

    pub fn with_identity_named(self, name: &str) -> Self {
        self.with_identity(IdentityPicker::Named(name.to_string()))
    }

    pub fn with_identity(self, identity: IdentityPicker) -> Self {
        Self { identity, ..self }
    }

    pub fn mainnet(self) -> Self {
        self.with_network(NetworkPicker::Mainnet)
    }

    pub fn with_network(self, network: NetworkPicker) -> Self {
        Self { network, ..self }
    }

    pub fn with_force_fetch_root_key_insecure_non_mainnet_only(
        self,
        force_fetch_root_key_insecure_non_mainnet_only: bool,
    ) -> Self {
        Self {
            force_fetch_root_key_insecure_non_mainnet_only,
            ..self
        }
    }

    pub async fn build(&self) -> Result<DfxInterface, BuildDfxInterfaceError> {
        let fetch_root_key = self.network == NetworkPicker::Local
            || self.force_fetch_root_key_insecure_non_mainnet_only;
        let networks_config = NetworksConfig::new()?;
        let config = Config::from_current_dir(None)?.map(Arc::new);
        let network_descriptor = self.build_network_descriptor(config.clone(), &networks_config)?;
        let identity = self.build_identity()?;
        let agent = self.build_agent(identity.clone(), &network_descriptor)?;

        if fetch_root_key {
            fetch_root_key_when_non_mainnet_or_error(&agent, &network_descriptor).await?;
        }

        Ok(DfxInterface {
            config,
            identity,
            agent,
            networks_config,
            network_descriptor,
        })
    }

    fn build_agent(
        &self,
        identity: Arc<dyn Identity>,
        network_descriptor: &NetworkDescriptor,
    ) -> Result<Agent, BuildAgentError> {
        let route_provider = RoundRobinRouteProvider::new(network_descriptor.providers.clone())
            .map_err(BuildAgentError::CreateRouteProvider)?;
        let client = Client::builder()
            .use_rustls_tls()
            .build()
            .map_err(BuildAgentError::CreateHttpClient)?;
        let transport = ReqwestHttpReplicaV2Transport::create_with_client_route(
            Arc::new(route_provider),
            client,
        )
        .map_err(BuildAgentError::CreateTransport)?;
        let agent = Agent::builder()
            .with_transport(transport)
            .with_arc_identity(identity)
            .build()
            .map_err(BuildAgentError::CreateAgent)?;
        Ok(agent)
    }

    fn build_identity(&self) -> Result<Arc<dyn Identity>, BuildIdentityError> {
        if self.identity == IdentityPicker::Anonymous {
            return Ok(Arc::new(ic_agent::identity::AnonymousIdentity));
        }

        let identity_override = match &self.identity {
            IdentityPicker::Named(name) => Some(name.clone()),
            IdentityPicker::Selected => None,
            IdentityPicker::Anonymous => unreachable!(),
        };

        let logger = slog::Logger::root(slog::Discard, slog::o!());
        let mut identity_manager =
            IdentityManager::new(&logger, identity_override.as_deref(), InitializeIdentity::Disallow)?;
        let identity: Box<dyn Identity> =
            identity_manager.instantiate_selected_identity(&logger)?;
        Ok(Arc::from(identity))
    }

    fn build_network_descriptor(
        &self,
        config: Option<Arc<Config>>,
        networks_config: &NetworksConfig,
    ) -> Result<NetworkDescriptor, NetworkConfigError> {
        let network = match &self.network {
            NetworkPicker::Local => None,
            NetworkPicker::Mainnet => Some("ic".to_string()),
            NetworkPicker::Named(name) => Some(name.clone()),
        }
        .map(String::from);
        let logger = None;
        create_network_descriptor(
            config,
            Arc::new(networks_config.clone()),
            network,
            logger,
            LocalBindDetermination::ApplyRunningWebserverPort,
        )
    }
}

impl Default for DfxInterfaceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;
    use candid::Principal;
    use ic_agent::Identity;
    use tempfile::TempDir;
    use crate::DfxInterface;
    use crate::error::builder::BuildDfxInterfaceError;
    use crate::error::builder::BuildIdentityError::NewIdentityManager;
    use crate::error::identity::new_identity_manager::NewIdentityManagerError;

    #[tokio::test]
    async fn anonymous() {
        // anonymous identity creates a .config directory, but doesn't put anything in it
        let td = setup();

        let d = DfxInterface::anonymous().await.unwrap();
        assert!(matches!(d.identity().public_key(), None));
        assert_eq!(d.identity().sender().unwrap(), Principal::anonymous());

        let actual = all_children(td.path());
        //  creates config directories, but they are empty
        //let expected: Vec<String> = vec!();
        let expected: Vec<String> = vec!(
            ".config/".into(),
            ".config/dfx/".into());
        assert_eq!(actual, expected);
        // let config_only: Vec<OsString> = vec!(".config".into());
        // let dfx_only: Vec<OsString> = vec!("dfx".into());
        // let empty: Vec<OsString> = vec!();
        //
        // assert_eq!(all_filenames(td.path()), config_only);
        // assert_eq!(all_filenames(&td.path().join(".config")), dfx_only);
        // assert_eq!(all_filenames(&td.path().join(".config").join("dfx")), empty);
    }

    #[tokio::test]
    async fn default_no_config() {
        let td = setup();
        assert!(matches!(DfxInterface::builder().build().await,
            Err(BuildDfxInterfaceError::BuildIdentity(
                NewIdentityManager(
                    NewIdentityManagerError::NoIdentityConfigurationFound)))));
    }

    fn all_filenames(dir: &Path) -> Vec<OsString> {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect()
    }

    fn all_children(dir: &Path) -> Vec<String> {
        eprintln!("all_pathnames({:?})", dir);
        // like find: recurse into subdirectories
        let mut result = vec![];
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let filename = path.file_name().unwrap().to_os_string().into_string().unwrap();

            if path.is_dir() {
                result.push(filename.clone() + "/");
                let all_children = all_children(&path);
                let all_children: Vec<_> = all_children.iter().map(|c| {
                    format!("{}/{}", &filename, c)
                }).collect();
                result.extend(all_children);
            } else {
                result.push(filename);

            }
        }
        eprintln!("all_pathnames({:?}) = {:?}", dir, result);
        result
    }

    fn setup() -> TempDir {
        let td = TempDir::new().unwrap();
        std::env::set_var("DFX_CONFIG_ROOT", td.path());
        td
    }
}