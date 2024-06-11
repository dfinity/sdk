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
    agent::http_transport::{route_provider::RoundRobinRouteProvider, ReqwestTransport},
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

    pub fn with_network_named(self, name: &str) -> Self {
        self.with_network(NetworkPicker::Named(name.to_string()))
    }

    pub fn with_force_fetch_root_key_insecure_non_mainnet_only(self) -> Self {
        Self {
            force_fetch_root_key_insecure_non_mainnet_only: true,
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
        let transport =
            ReqwestTransport::create_with_client_route(Arc::new(route_provider), client)
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
        let mut identity_manager = IdentityManager::new(
            &logger,
            identity_override.as_deref(),
            InitializeIdentity::Disallow,
        )?;
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
    use crate::error::{
        builder::{
            BuildDfxInterfaceError,
            BuildDfxInterfaceError::{FetchRootKey, NetworkConfig},
            BuildIdentityError::NewIdentityManager,
        },
        identity::new_identity_manager::NewIdentityManagerError,
        network_config::NetworkConfigError::NetworkNotFound,
        root_key::FetchRootKeyError,
        root_key::FetchRootKeyError::AgentError,
    };
    use crate::identity::{
        identity_manager::IdentityStorageMode::Plaintext, IdentityCreationParameters,
        IdentityManager,
    };
    use crate::DfxInterface;
    use candid::Principal;
    use futures::Future;
    use ic_agent::{AgentError::TransportError, Identity};
    use serde_json::json;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Semaphore;

    lazy_static::lazy_static! {
        static ref SEMAPHORE: Semaphore = Semaphore::new(1);
    }

    async fn run_test<F, Fut>(test_function: F)
    where
        F: FnOnce(Arc<TempDir>) -> Fut + Send,
        Fut: Future<Output = ()> + Send,
    {
        run_test_with_settings(TestSettings { testnet: None }, test_function).await;
    }

    pub struct TestNetSettings {
        pub name: String,
        pub providers: Vec<String>,
    }
    pub struct TestSettings {
        pub testnet: Option<TestNetSettings>,
    }
    async fn run_test_with_settings<F, Fut>(settings: TestSettings, test_function: F)
    where
        F: FnOnce(Arc<TempDir>) -> Fut + Send,
        Fut: Future<Output = ()> + Send,
    {
        let _permit = SEMAPHORE.acquire().await.unwrap();

        let temp_dir = TempDir::new().unwrap();

        if let Some(testnet) = settings.testnet {
            let networks_config = json!({
                testnet.name: {
                    "providers": testnet.providers,
                }
            });

            let config_dir = temp_dir.path().join(".config/dfx");
            let networks_config_path = config_dir.join("networks.json");
            crate::fs::create_dir_all(&config_dir).unwrap();
            crate::fs::write(networks_config_path, networks_config.to_string()).unwrap();
        }

        // so tests don't clobber each other in the environment
        std::env::set_var("DFX_CONFIG_ROOT", temp_dir.path());

        let temp_dir = Arc::new(temp_dir);
        test_function(temp_dir.clone()).await;
    }

    #[tokio::test]
    async fn anonymous() {
        run_test(|td| async move {
            let d = DfxInterface::builder()
                .anonymous()
                .mainnet()
                .build()
                .await
                .unwrap();
            assert!(d.identity().public_key().is_none());
            assert_eq!(d.identity().sender().unwrap(), Principal::anonymous());

            let actual = all_children(td.path());
            let expected: Vec<String> = vec![".config/".into(), ".config/dfx/".into()];
            assert_eq!(actual, expected);
        })
        .await;
    }

    #[tokio::test]
    async fn no_config_does_not_create_default_identity() {
        run_test(|_| async {
            assert!(matches!(
                DfxInterface::builder().build().await,
                Err(BuildDfxInterfaceError::BuildIdentity(NewIdentityManager(
                    NewIdentityManagerError::NoIdentityConfigurationFound
                )))
            ));
        })
        .await;
    }

    #[tokio::test]
    async fn default_identity() {
        run_test(|_| async {
            let default_principal = {
                let logger = slog::Logger::root(slog::Discard, slog::o!());
                let mut im = IdentityManager::new(
                    &logger,
                    None,
                    crate::identity::identity_manager::InitializeIdentity::Allow,
                )
                .unwrap();
                let id: Box<dyn Identity> = im.instantiate_selected_identity(&logger).unwrap();
                id.sender().unwrap()
            };
            let d = DfxInterface::builder().mainnet().build().await.unwrap();
            assert_eq!(d.identity.sender().unwrap(), default_principal);
        })
        .await;
    }

    #[tokio::test]
    async fn select_identity_by_name() {
        run_test(|_| async {
            let alice = "alice";
            let bob = "bob";
            let (alice_principal_from_mgr, bob_principal_from_mgr) = {
                let logger = slog::Logger::root(slog::Discard, slog::o!());
                let mut im = IdentityManager::new(
                    &logger,
                    None,
                    crate::identity::identity_manager::InitializeIdentity::Allow,
                )
                .unwrap();
                im.create_new_identity(&logger, alice, plaintext(), false)
                    .unwrap();
                im.create_new_identity(&logger, bob, plaintext(), false)
                    .unwrap();

                let alice: Box<dyn Identity> =
                    im.instantiate_identity_from_name(alice, &logger).unwrap();

                let bob: Box<dyn Identity> =
                    im.instantiate_identity_from_name(bob, &logger).unwrap();
                (alice.sender().unwrap(), bob.sender().unwrap())
            };
            assert_ne!(alice_principal_from_mgr, bob_principal_from_mgr);
            let alice_interface = DfxInterface::builder()
                .with_identity_named(alice)
                .mainnet()
                .build()
                .await
                .unwrap();
            assert_eq!(
                alice_interface.identity.sender().unwrap(),
                alice_principal_from_mgr
            );

            let bob_interface = DfxInterface::builder()
                .with_identity_named(bob)
                .mainnet()
                .build()
                .await
                .unwrap();
            assert_eq!(
                bob_interface.identity.sender().unwrap(),
                bob_principal_from_mgr
            );
        })
        .await;
    }

    #[tokio::test]
    async fn selected_non_default() {
        run_test(|_| async {
            let alice = "alice";
            let bob = "bob";
            let bob_principal_from_mgr = {
                let logger = slog::Logger::root(slog::Discard, slog::o!());
                let mut im = IdentityManager::new(
                    &logger,
                    None,
                    crate::identity::identity_manager::InitializeIdentity::Allow,
                )
                .unwrap();
                im.create_new_identity(&logger, alice, plaintext(), false)
                    .unwrap();
                im.create_new_identity(&logger, bob, plaintext(), false)
                    .unwrap();

                let _alice_identity: Box<dyn Identity> =
                    im.instantiate_identity_from_name(alice, &logger).unwrap();

                let bob_identity: Box<dyn Identity> =
                    im.instantiate_identity_from_name(bob, &logger).unwrap();

                im.use_identity_named(&logger, bob).unwrap();
                bob_identity.sender().unwrap()
            };

            let selected_interface = DfxInterface::builder().mainnet().build().await.unwrap();
            assert_eq!(
                selected_interface.identity.sender().unwrap(),
                bob_principal_from_mgr
            );
        })
        .await;
    }

    fn plaintext() -> IdentityCreationParameters {
        IdentityCreationParameters::Pem { mode: Plaintext }
    }

    #[tokio::test]
    async fn local_network() {
        run_test(|_| async {
            match DfxInterface::builder().anonymous().build().await {
                Ok(d) => {
                    assert_eq!(d.network_descriptor.name, "local");
                    assert!(!d.network_descriptor.is_ic);
                    assert!(d.network_descriptor.local_server_descriptor.is_some());
                }
                Err(FetchRootKey(AgentError(TransportError(_)))) => {
                    // local replica isn't running, so this is expected,
                    // but we can't check anything else
                }
                Err(e) => panic!("unexpected error: {:?}", e),
            }
        })
        .await;
    }

    #[tokio::test]
    async fn mainnet() {
        run_test(|_| async {
            let d = DfxInterface::builder()
                .anonymous()
                .mainnet()
                .build()
                .await
                .unwrap();
            let network_descriptor = d.network_descriptor;
            assert!(network_descriptor.is_ic);
            assert_eq!(network_descriptor.name, "ic");
        })
        .await;
    }

    #[tokio::test]
    async fn try_to_fetch_root_key_on_mainnet() {
        run_test(|_| async {
            assert!(matches!(
                DfxInterface::builder()
                    .anonymous()
                    .mainnet()
                    .with_force_fetch_root_key_insecure_non_mainnet_only()
                    .build()
                    .await,
                Err(FetchRootKey(
                    FetchRootKeyError::MustNotFetchRootKeyOnMainnet
                ))
            ));
        })
        .await;
    }

    #[tokio::test]
    async fn named_network_not_found() {
        run_test(|_| async {
            assert!(
                matches!(DfxInterface::builder().with_network_named("testnet").build().await,
                Err(NetworkConfig(NetworkNotFound(network_name))) if network_name == "testnet")
            );
        })
        .await;
    }

    #[tokio::test]
    async fn named_network() {
        let settings = TestSettings {
            testnet: Some(TestNetSettings {
                name: "testnet".to_string(),
                providers: vec!["http://localhost:1234".to_string()],
            }),
        };
        run_test_with_settings(settings, |_| async move {
            let d = DfxInterface::builder()
                .anonymous()
                .with_network_named("testnet")
                .build()
                .await
                .unwrap();
            let network_descriptor = d.network_descriptor;
            assert_eq!(network_descriptor.name, "testnet");
            assert_eq!(network_descriptor.providers, vec!["http://localhost:1234"]);

            // Notice that the above did not fail, because it did not try to fetch the root key.
            // It only does so if we tell it to:
            assert!(matches!(
                DfxInterface::builder()
                    .anonymous()
                    .with_network_named("testnet")
                    .with_force_fetch_root_key_insecure_non_mainnet_only()
                    .build()
                    .await,
                Err(FetchRootKey(AgentError(TransportError(_))))
            ));
        })
        .await;
    }

    // returns a vec of all children of a directory, recursively
    // directories are suffixed with a '/'
    fn all_children(dir: &Path) -> Vec<String> {
        let mut result = vec![];
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let filename = path
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap();

            if path.is_dir() {
                result.push(filename.clone() + "/");
                let all_children = all_children(&path);
                let all_children: Vec<_> = all_children
                    .iter()
                    .map(|c| format!("{}/{}", &filename, c))
                    .collect();
                result.extend(all_children);
            } else {
                result.push(filename);
            }
        }
        result
    }
}
