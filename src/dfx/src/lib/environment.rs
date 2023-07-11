use crate::config::cache::DiskBasedCache;
use crate::config::dfx_version;
use crate::lib::error::extension::ExtensionError;
use crate::lib::error::DfxResult;
use crate::lib::extension::manager::ExtensionManager;
use crate::lib::progress_bar::ProgressBar;
use crate::lib::warning::{is_warning_disabled, DfxWarning::MainnetPlainTextIdentity};
use dfx_core::config::cache::Cache;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::dfinity::{Config, NetworksConfig};
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::error::canister_id_store::CanisterIdStoreError;
use dfx_core::error::identity::IdentityError;
use dfx_core::identity::identity_manager::IdentityManager;

use anyhow::{anyhow, Context};
use candid::Principal;
use fn_error_context::context;
use ic_agent::{Agent, Identity};
use semver::Version;
use slog::{warn, Logger, Record};
use std::borrow::Cow;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub trait Environment {
    fn get_cache(&self) -> Arc<dyn Cache>;
    fn get_config(&self) -> Option<Arc<Config>>;
    fn get_networks_config(&self) -> Arc<NetworksConfig>;
    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>>;

    fn is_in_project(&self) -> bool;
    /// Return a temporary directory for the current project.
    /// If there is no project (no dfx.json), there is no project temp dir.
    fn get_project_temp_dir(&self) -> Option<PathBuf>;

    fn get_version(&self) -> &Version;

    /// This is value of the name passed to dfx `--identity <name>`
    /// Notably, it is _not_ the name of the default identity or selected identity
    fn get_identity_override(&self) -> &Option<String>;

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn get_agent<'a>(&'a self) -> Option<&'a Agent>;

    #[allow(clippy::needless_lifetimes)]
    fn get_network_descriptor<'a>(&'a self) -> &'a NetworkDescriptor;

    fn get_logger(&self) -> &slog::Logger;
    fn get_verbose_level(&self) -> i64;
    fn new_spinner(&self, message: Cow<'static, str>) -> ProgressBar;
    fn new_progress(&self, message: &str) -> ProgressBar;

    fn new_identity_manager(&self) -> Result<IdentityManager, IdentityError> {
        IdentityManager::new(self.get_logger(), self.get_identity_override())
    }

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn log<'a>(&self, record: &Record<'a>) {
        self.get_logger().log(record);
    }

    fn get_selected_identity(&self) -> Option<&String>;

    fn get_selected_identity_principal(&self) -> Option<Principal>;

    fn get_effective_canister_id(&self) -> Principal;

    fn new_extension_manager(&self) -> Result<ExtensionManager, ExtensionError>;

    fn get_canister_id_store(&self) -> Result<CanisterIdStore, CanisterIdStoreError> {
        CanisterIdStore::new(self.get_network_descriptor(), self.get_config())
    }
}

pub struct EnvironmentImpl {
    config: Option<Arc<Config>>,
    shared_networks_config: Arc<NetworksConfig>,

    cache: Arc<dyn Cache>,

    version: Version,

    logger: Option<slog::Logger>,
    verbose_level: i64,

    identity_override: Option<String>,

    effective_canister_id: Principal,
}

impl EnvironmentImpl {
    pub fn new() -> DfxResult<Self> {
        let shared_networks_config = NetworksConfig::new()?;
        let config = Config::from_current_dir()?;
        if let Some(ref config) = config {
            let temp_dir = config.get_temp_path();
            create_dir_all(&temp_dir).with_context(|| {
                format!("Failed to create temp directory {}.", temp_dir.display())
            })?;
        }

        // Figure out which version of DFX we should be running. This will use the following
        // fallback sequence:
        //   1. DFX_VERSION environment variable
        //   2. dfx.json "dfx" field
        //   3. this binary's version
        // If any of those are empty string, we stop the fallback and use the current version.
        // If any of those are a valid version, we try to use that directly as is.
        // If any of those are an invalid version, we will show an error to the user.
        let version = match std::env::var("DFX_VERSION") {
            Err(_) => match &config {
                None => dfx_version().clone(),
                Some(c) => match &c.get_config().get_dfx() {
                    None => dfx_version().clone(),
                    Some(v) => Version::parse(v)
                        .with_context(|| format!("Failed to parse version from '{}'.", v))?,
                },
            },
            Ok(v) => {
                if v.is_empty() {
                    dfx_version().clone()
                } else {
                    Version::parse(&v)
                        .with_context(|| format!("Failed to parse version from '{}'.", &v))?
                }
            }
        };

        Ok(EnvironmentImpl {
            cache: Arc::new(DiskBasedCache::with_version(&version)),
            config: config.map(Arc::new),
            shared_networks_config: Arc::new(shared_networks_config),
            version: version.clone(),
            logger: None,
            verbose_level: 0,
            identity_override: None,
            effective_canister_id: Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 1, 1]),
        })
    }

    pub fn with_logger(mut self, logger: slog::Logger) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn with_identity_override(mut self, identity: Option<String>) -> Self {
        self.identity_override = identity;
        self
    }

    pub fn with_verbose_level(mut self, verbose_level: i64) -> Self {
        self.verbose_level = verbose_level;
        self
    }

    pub fn with_effective_canister_id(mut self, effective_canister_id: Option<String>) -> Self {
        match effective_canister_id {
            None => self,
            Some(canister_id) => match Principal::from_text(canister_id) {
                Ok(principal) => {
                    self.effective_canister_id = principal;
                    self
                }
                Err(_) => self,
            },
        }
    }
}

impl Environment for EnvironmentImpl {
    fn get_cache(&self) -> Arc<dyn Cache> {
        Arc::clone(&self.cache)
    }

    fn get_config(&self) -> Option<Arc<Config>> {
        self.config.as_ref().map(Arc::clone)
    }

    fn get_networks_config(&self) -> Arc<NetworksConfig> {
        self.shared_networks_config.clone()
    }

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config().ok_or_else(|| anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn is_in_project(&self) -> bool {
        self.config.is_some()
    }

    fn get_project_temp_dir(&self) -> Option<PathBuf> {
        self.config.as_ref().map(|c| c.get_temp_path())
    }

    fn get_version(&self) -> &Version {
        &self.version
    }

    fn get_identity_override(&self) -> &Option<String> {
        &self.identity_override
    }

    fn get_agent(&self) -> Option<&Agent> {
        // create an AgentEnvironment explicitly, in order to specify network and agent.
        // See install, build for examples.
        None
    }

    fn get_network_descriptor(&self) -> &NetworkDescriptor {
        // It's not valid to call get_network_descriptor on an EnvironmentImpl.
        // All of the places that call this have an AgentEnvironment anyway.
        unreachable!("NetworkDescriptor only available from an AgentEnvironment");
    }

    fn get_logger(&self) -> &slog::Logger {
        self.logger
            .as_ref()
            .expect("Log was not setup, but is being used.")
    }

    fn get_verbose_level(&self) -> i64 {
        self.verbose_level
    }

    fn new_spinner(&self, message: Cow<'static, str>) -> ProgressBar {
        // Only show the progress bar if the level is INFO or more.
        if self.verbose_level >= 0 {
            ProgressBar::new_spinner(message)
        } else {
            ProgressBar::discard()
        }
    }

    fn new_progress(&self, _message: &str) -> ProgressBar {
        ProgressBar::discard()
    }

    fn get_selected_identity(&self) -> Option<&String> {
        None
    }

    fn get_selected_identity_principal(&self) -> Option<Principal> {
        None
    }

    fn get_effective_canister_id(&self) -> Principal {
        self.effective_canister_id
    }

    fn new_extension_manager(&self) -> Result<ExtensionManager, ExtensionError> {
        ExtensionManager::new(self.get_version())
    }
}

pub struct AgentEnvironment<'a> {
    backend: &'a dyn Environment,
    agent: Agent,
    network_descriptor: NetworkDescriptor,
    identity_manager: IdentityManager,
}

impl<'a> AgentEnvironment<'a> {
    #[context("Failed to create AgentEnvironment for network '{}'.", network_descriptor.name)]
    pub fn new(
        backend: &'a dyn Environment,
        network_descriptor: NetworkDescriptor,
        timeout: Duration,
        use_identity: Option<&str>,
    ) -> DfxResult<Self> {
        let logger = backend.get_logger().clone();
        let mut identity_manager = backend.new_identity_manager()?;
        let identity = if let Some(identity_name) = use_identity {
            identity_manager.instantiate_identity_from_name(identity_name, &logger)?
        } else {
            identity_manager.instantiate_selected_identity(&logger)?
        };
        if network_descriptor.is_ic
            && identity.insecure
            && !is_warning_disabled(MainnetPlainTextIdentity)
        {
            warn!(logger, "The {} identity is not stored securely. Do not use it to control a lot of cycles/ICP. Create a new identity with `dfx identity new` \
                and use it in mainnet-facing commands with the `--identity` flag", identity.name());
        }
        let url = network_descriptor.first_provider()?;

        Ok(AgentEnvironment {
            backend,
            agent: create_agent(logger, url, identity, timeout)?,
            network_descriptor: network_descriptor.clone(),
            identity_manager,
        })
    }
}

impl<'a> Environment for AgentEnvironment<'a> {
    fn get_cache(&self) -> Arc<dyn Cache> {
        self.backend.get_cache()
    }

    fn get_config(&self) -> Option<Arc<Config>> {
        self.backend.get_config()
    }

    fn get_networks_config(&self) -> Arc<NetworksConfig> {
        self.backend.get_networks_config()
    }

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config().ok_or_else(|| anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn is_in_project(&self) -> bool {
        self.backend.is_in_project()
    }

    fn get_project_temp_dir(&self) -> Option<PathBuf> {
        self.backend.get_project_temp_dir()
    }

    fn get_version(&self) -> &Version {
        self.backend.get_version()
    }

    fn get_identity_override(&self) -> &Option<String> {
        self.backend.get_identity_override()
    }

    fn get_agent(&self) -> Option<&Agent> {
        Some(&self.agent)
    }

    fn get_network_descriptor(&self) -> &NetworkDescriptor {
        &self.network_descriptor
    }

    fn get_logger(&self) -> &slog::Logger {
        self.backend.get_logger()
    }

    fn get_verbose_level(&self) -> i64 {
        self.backend.get_verbose_level()
    }

    fn new_spinner(&self, message: Cow<'static, str>) -> ProgressBar {
        self.backend.new_spinner(message)
    }

    fn new_progress(&self, message: &str) -> ProgressBar {
        self.backend.new_progress(message)
    }

    fn get_selected_identity(&self) -> Option<&String> {
        Some(self.identity_manager.get_selected_identity_name())
    }

    fn get_selected_identity_principal(&self) -> Option<Principal> {
        self.identity_manager.get_selected_identity_principal()
    }

    fn get_effective_canister_id(&self) -> Principal {
        self.backend.get_effective_canister_id()
    }

    fn new_extension_manager(&self) -> Result<ExtensionManager, ExtensionError> {
        ExtensionManager::new(self.backend.get_version())
    }
}

#[context("Failed to create agent with url {}.", url)]
pub fn create_agent(
    _logger: Logger,
    url: &str,
    identity: Box<dyn Identity + Send + Sync>,
    timeout: Duration,
) -> DfxResult<Agent> {
    let agent = Agent::builder()
        .with_transport(ic_agent::agent::http_transport::ReqwestTransport::create(
            url,
        )?)
        .with_boxed_identity(identity)
        .with_ingress_expiry(Some(timeout))
        .build()?;
    Ok(agent)
}
