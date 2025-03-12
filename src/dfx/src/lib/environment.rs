use crate::config::cache::VersionCache;
use crate::config::dfx_version;
use crate::lib::error::DfxResult;
use crate::lib::progress_bar::ProgressBar;
use crate::lib::warning::{is_warning_disabled, DfxWarning::MainnetPlainTextIdentity};
use anyhow::{anyhow, bail};
use candid::Principal;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::dfinity::{Config, NetworksConfig, TelemetryState, ToolConfig};
use dfx_core::config::model::network_descriptor::{NetworkDescriptor, NetworkTypeDescriptor};
use dfx_core::error::canister_id_store::CanisterIdStoreError;
use dfx_core::error::identity::NewIdentityManagerError;
use dfx_core::error::load_dfx_config::LoadDfxConfigError;
use dfx_core::error::uri::UriError;
use dfx_core::extension::manager::ExtensionManager;
use dfx_core::identity::identity_manager::{IdentityManager, InitializeIdentity};
use fn_error_context::context;
use ic_agent::{Agent, Identity};
use indicatif::MultiProgress;
use pocket_ic::nonblocking::PocketIc;
use semver::Version;
use slog::{Logger, Record};
use std::borrow::Cow;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

pub trait Environment {
    fn get_cache(&self) -> VersionCache;
    fn get_config(&self) -> Result<Option<Arc<Config>>, LoadDfxConfigError>;
    fn get_networks_config(&self) -> Arc<NetworksConfig>;
    fn get_tool_config(&self) -> Arc<Mutex<ToolConfig>>;
    fn telemetry_mode(&self) -> TelemetryState;
    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>>;

    /// Return a temporary directory for the current project.
    /// If there is no project (no dfx.json), there is no project temp dir.
    fn get_project_temp_dir(&self) -> DfxResult<Option<PathBuf>>;

    fn get_version(&self) -> &Version;

    /// This is value of the name passed to dfx `--identity <name>`
    /// Notably, it is _not_ the name of the default identity or selected identity
    fn get_identity_override(&self) -> Option<&str>;

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn get_agent<'a>(&'a self) -> &'a Agent;

    fn get_pocketic(&self) -> Option<&PocketIc>;

    #[allow(clippy::needless_lifetimes)]
    fn get_network_descriptor<'a>(&'a self) -> &'a NetworkDescriptor;

    fn get_logger(&self) -> &slog::Logger;
    fn get_verbose_level(&self) -> i64;
    fn new_spinner(&self, message: Cow<'static, str>) -> ProgressBar;
    fn with_suspend_all_spinners(&self, f: Box<dyn FnOnce() + '_>); // box needed for dyn Environment
    fn new_progress(&self, message: &str) -> ProgressBar;

    fn new_identity_manager(&self) -> Result<IdentityManager, NewIdentityManagerError> {
        IdentityManager::new(
            self.get_logger(),
            self.get_identity_override(),
            InitializeIdentity::Allow,
        )
    }

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes, unused)]
    fn log<'a>(&self, record: &Record<'a>) {
        self.get_logger().log(record);
    }

    fn get_selected_identity(&self) -> Option<&String>;

    fn get_selected_identity_principal(&self) -> Option<Principal>;

    fn get_effective_canister_id(&self) -> Principal;

    fn get_override_effective_canister_id(&self) -> Option<Principal>;

    fn get_extension_manager(&self) -> &ExtensionManager;

    fn get_canister_id_store(&self) -> Result<CanisterIdStore, CanisterIdStoreError> {
        CanisterIdStore::new(
            self.get_logger(),
            self.get_network_descriptor(),
            self.get_config()?,
        )
    }
}

pub enum ProjectConfig {
    NotLoaded,
    NoProject,
    Loaded(Arc<Config>),
}

pub struct EnvironmentImpl {
    project_config: RefCell<ProjectConfig>,
    shared_networks_config: Arc<NetworksConfig>,
    tool_config: Arc<Mutex<ToolConfig>>,

    cache: VersionCache,

    version: Version,

    logger: Option<slog::Logger>,
    verbose_level: i64,

    spinners: MultiProgress,

    identity_override: Option<String>,

    effective_canister_id: Option<Principal>,

    extension_manager: ExtensionManager,
}

impl EnvironmentImpl {
    pub fn new(extension_manager: ExtensionManager, tool_config: ToolConfig) -> DfxResult<Self> {
        let shared_networks_config = NetworksConfig::new()?;
        let version = dfx_version().clone();

        Ok(EnvironmentImpl {
            cache: VersionCache::with_version(&version),
            project_config: RefCell::new(ProjectConfig::NotLoaded),
            shared_networks_config: Arc::new(shared_networks_config),
            tool_config: Arc::new(Mutex::new(tool_config)),
            version: version.clone(),
            logger: None,
            verbose_level: 0,
            identity_override: None,
            effective_canister_id: None,
            extension_manager,
            spinners: MultiProgress::new(),
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
            None => {
                self.effective_canister_id = None;
                self
            }
            Some(canister_id) => match Principal::from_text(canister_id) {
                Ok(principal) => {
                    self.effective_canister_id = Some(principal);
                    self
                }
                Err(_) => self,
            },
        }
    }

    pub fn with_spinners(mut self, spinners: MultiProgress) -> Self {
        self.spinners = spinners;
        self
    }

    fn load_config(&self) -> Result<(), LoadDfxConfigError> {
        let config = Config::from_current_dir(Some(&self.extension_manager))?;

        let project_config = config.map_or(ProjectConfig::NoProject, |config| {
            ProjectConfig::Loaded(Arc::new(config))
        });
        self.project_config.replace(project_config);
        Ok(())
    }
}

impl Environment for EnvironmentImpl {
    fn get_cache(&self) -> VersionCache {
        self.cache.clone()
    }

    fn get_config(&self) -> Result<Option<Arc<Config>>, LoadDfxConfigError> {
        if matches!(*self.project_config.borrow(), ProjectConfig::NotLoaded) {
            self.load_config()?;
        }

        let config = if let ProjectConfig::Loaded(ref config) = *self.project_config.borrow() {
            Some(Arc::clone(config))
        } else {
            None
        };
        Ok(config)
    }

    fn get_networks_config(&self) -> Arc<NetworksConfig> {
        self.shared_networks_config.clone()
    }

    fn get_tool_config(&self) -> Arc<Mutex<ToolConfig>> {
        self.tool_config.clone()
    }

    fn telemetry_mode(&self) -> TelemetryState {
        if let Ok(var) = std::env::var("DFX_TELEMETRY") {
            if !var.is_empty() {
                return match &*var {
                    "true" | "1" | "on" => TelemetryState::On,
                    "false" | "0" | "off" => TelemetryState::Off,
                    "local" => TelemetryState::Local,
                    _ => TelemetryState::On,
                };
            }
        }
        self.tool_config.lock().unwrap().interface().telemetry
    }

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config()?.ok_or_else(|| anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn get_project_temp_dir(&self) -> DfxResult<Option<PathBuf>> {
        Ok(self.get_config()?.map(|c| c.get_temp_path()).transpose()?)
    }

    fn get_version(&self) -> &Version {
        &self.version
    }

    fn get_identity_override(&self) -> Option<&str> {
        self.identity_override.as_deref()
    }

    fn get_agent(&self) -> &Agent {
        unreachable!("Agent only available from an AgentEnvironment");
    }

    fn get_pocketic(&self) -> Option<&PocketIc> {
        unreachable!("PocketIC handle only available from an AgentEnvironment");
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
            ProgressBar::new_spinner(message, &self.spinners)
        } else {
            ProgressBar::discard()
        }
    }

    fn with_suspend_all_spinners(&self, f: Box<dyn FnOnce() + '_>) {
        self.spinners.suspend(f);
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
            .unwrap_or(Principal::from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 1, 1]))
    }

    fn get_override_effective_canister_id(&self) -> Option<Principal> {
        self.effective_canister_id
    }

    fn get_extension_manager(&self) -> &ExtensionManager {
        &self.extension_manager
    }
}

pub struct AgentEnvironment<'a> {
    backend: &'a dyn Environment,
    agent: Agent,
    pocketic: Option<PocketIc>,
    network_descriptor: NetworkDescriptor,
    identity_manager: IdentityManager,
    effective_canister_id: Option<Principal>,
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
            && !matches!(
                network_descriptor.r#type,
                NetworkTypeDescriptor::Playground { .. }
            )
            && identity.insecure
            && !is_warning_disabled(MainnetPlainTextIdentity)
        {
            bail!(
                "The {} identity is not stored securely. Do not use it to control a lot of cycles/ICP.
- For enhanced security, create a new identity using the command: 
    dfx identity new
  Then, specify the new identity in mainnet-facing commands with the `--identity` flag.
- If you understand the risks and still wish to use the insecure plaintext identity, you can suppress this warning by running:
    export DFX_WARNING=-mainnet_plaintext_identity
  After setting this environment variable, re-run the command.",
                identity.name()
            );
        }
        let url = network_descriptor.first_provider()?;
        let effective_canister_id = if let Some(d) = &network_descriptor.local_server_descriptor {
            d.effective_config()?
                .and_then(|c| c.get_effective_canister_id())
        } else {
            None
        };

        let pocketic =
            if let Some(local_server_descriptor) = &network_descriptor.local_server_descriptor {
                match local_server_descriptor.get_running_pocketic_port(None)? {
                    Some(port) => {
                        let mut socket_addr = local_server_descriptor.bind_address;
                        socket_addr.set_port(port);
                        let url = format!("http://{}", socket_addr);
                        let url = Url::parse(&url)
                            .map_err(|e| UriError::UrlParseError(url.to_string(), e))?;
                        Some(create_pocketic(&url))
                    }
                    None => None,
                }
            } else {
                None
            };

        Ok(AgentEnvironment {
            backend,
            agent: create_agent(logger, url, identity, timeout)?,
            pocketic,
            network_descriptor: network_descriptor.clone(),
            identity_manager,
            effective_canister_id,
        })
    }
}

impl<'a> Environment for AgentEnvironment<'a> {
    fn get_cache(&self) -> VersionCache {
        self.backend.get_cache()
    }

    fn get_config(&self) -> Result<Option<Arc<Config>>, LoadDfxConfigError> {
        self.backend.get_config()
    }

    fn get_tool_config(&self) -> Arc<Mutex<ToolConfig>> {
        self.backend.get_tool_config()
    }

    fn telemetry_mode(&self) -> TelemetryState {
        self.backend.telemetry_mode()
    }

    fn get_networks_config(&self) -> Arc<NetworksConfig> {
        self.backend.get_networks_config()
    }

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config()?.ok_or_else(|| anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn get_project_temp_dir(&self) -> DfxResult<Option<PathBuf>> {
        self.backend.get_project_temp_dir()
    }

    fn get_version(&self) -> &Version {
        self.backend.get_version()
    }

    fn get_identity_override(&self) -> Option<&str> {
        self.backend.get_identity_override()
    }

    fn get_agent(&self) -> &Agent {
        &self.agent
    }

    fn get_pocketic(&self) -> Option<&PocketIc> {
        self.pocketic.as_ref()
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

    fn with_suspend_all_spinners(&self, f: Box<dyn FnOnce() + '_>) {
        self.backend.with_suspend_all_spinners(f);
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
        self.backend
            .get_override_effective_canister_id()
            .unwrap_or_else(|| {
                self.effective_canister_id
                    .unwrap_or_else(|| self.backend.get_effective_canister_id())
            })
    }

    fn get_override_effective_canister_id(&self) -> Option<Principal> {
        self.backend.get_override_effective_canister_id()
    }

    fn get_extension_manager(&self) -> &ExtensionManager {
        self.backend.get_extension_manager()
    }
}

#[context("Failed to create agent with url {}.", url)]
pub fn create_agent(
    _logger: Logger,
    url: &str,
    identity: Box<dyn Identity + Send + Sync>,
    timeout: Duration,
) -> DfxResult<Agent> {
    let disable_query_verification =
        std::env::var("DFX_DISABLE_QUERY_VERIFICATION").is_ok_and(|x| !x.trim().is_empty());
    let agent = Agent::builder()
        .with_url(url)
        .with_boxed_identity(identity)
        .with_verify_query_signatures(!disable_query_verification)
        .with_ingress_expiry(timeout)
        .build()?;
    Ok(agent)
}

pub fn create_pocketic(url: &Url) -> PocketIc {
    PocketIc::new_from_existing_instance(url.clone(), 0, None)
}

#[cfg(test)]
pub mod test_env {
    use super::*;

    /// Provides access to log-message-generating functions in test mode.
    pub struct TestEnv;
    impl Environment for TestEnv {
        fn get_agent(&self) -> &Agent {
            unimplemented!()
        }
        fn get_cache(&self) -> VersionCache {
            unimplemented!()
        }
        fn get_canister_id_store(&self) -> Result<CanisterIdStore, CanisterIdStoreError> {
            unimplemented!()
        }
        fn get_config(&self) -> Result<Option<Arc<Config>>, LoadDfxConfigError> {
            unimplemented!()
        }
        fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
            bail!("dummy env")
        }
        fn get_effective_canister_id(&self) -> Principal {
            unimplemented!()
        }
        fn get_extension_manager(&self) -> &ExtensionManager {
            unimplemented!()
        }
        fn get_identity_override(&self) -> Option<&str> {
            None
        }
        fn get_logger(&self) -> &slog::Logger {
            unimplemented!()
        }
        fn get_network_descriptor(&self) -> &NetworkDescriptor {
            unimplemented!()
        }
        fn get_networks_config(&self) -> Arc<NetworksConfig> {
            unimplemented!()
        }
        fn get_tool_config(&self) -> Arc<Mutex<ToolConfig>> {
            unimplemented!()
        }
        fn get_override_effective_canister_id(&self) -> Option<Principal> {
            None
        }
        fn get_pocketic(&self) -> Option<&PocketIc> {
            None
        }
        fn get_project_temp_dir(&self) -> DfxResult<Option<PathBuf>> {
            Ok(None)
        }
        fn get_selected_identity(&self) -> Option<&String> {
            unimplemented!()
        }
        fn get_selected_identity_principal(&self) -> Option<Principal> {
            unimplemented!()
        }
        fn get_verbose_level(&self) -> i64 {
            0
        }
        fn get_version(&self) -> &Version {
            unimplemented!()
        }
        fn telemetry_mode(&self) -> TelemetryState {
            TelemetryState::Off
        }
        fn log(&self, _record: &Record) {}
        fn new_identity_manager(&self) -> Result<IdentityManager, NewIdentityManagerError> {
            unimplemented!()
        }
        fn new_progress(&self, _message: &str) -> ProgressBar {
            ProgressBar::discard()
        }
        fn with_suspend_all_spinners(&self, f: Box<dyn FnOnce() + '_>) {
            f()
        }
        fn new_spinner(&self, _message: Cow<'static, str>) -> ProgressBar {
            ProgressBar::discard()
        }
    }
}
