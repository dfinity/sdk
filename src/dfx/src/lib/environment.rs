use crate::config::cache::{Cache, DiskBasedCache};
use crate::config::dfinity::{Config, NetworksConfig};
use crate::config::{cache, dfx_version};
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::progress_bar::ProgressBar;

use anyhow::{anyhow, Context};
use candid::Principal;
use fn_error_context::context;
use ic_agent::{Agent, Identity};
use semver::Version;
use slog::{Logger, Record};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn log<'a>(&self, record: &Record<'a>) {
        self.get_logger().log(record);
    }

    fn get_selected_identity(&self) -> Option<&String>;

    fn get_selected_identity_principal(&self) -> Option<Principal>;
}

pub struct EnvironmentImpl {
    config: Option<Arc<Config>>,
    shared_networks_config: Arc<NetworksConfig>,

    cache: Arc<dyn Cache>,

    version: Version,

    logger: Option<slog::Logger>,
    verbose_level: i64,

    identity_override: Option<String>,
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
    ) -> DfxResult<Self> {
        let logger = backend.get_logger().clone();
        let mut identity_manager = IdentityManager::new(backend)?;
        let identity = identity_manager.instantiate_selected_identity()?;
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
}

pub struct AgentClient {
    logger: Logger,
    url: reqwest::Url,

    // The auth `(username, password)`.
    auth: Arc<Mutex<Option<(String, String)>>>,
}

impl AgentClient {
    pub fn new(logger: Logger, url: String) -> DfxResult<AgentClient> {
        let url = reqwest::Url::parse(&url).with_context(|| format!("Invalid URL: {}", url))?;

        let result = Self {
            logger,
            url,
            auth: Arc::new(Mutex::new(None)),
        };

        if let Ok(Some(auth)) = result.read_http_auth() {
            result.auth.lock().unwrap().replace(auth);
        }

        Ok(result)
    }

    #[context("Failed to determine http auth path.")]
    fn http_auth_path() -> DfxResult<PathBuf> {
        Ok(cache::get_cache_root()?.join("http_auth"))
    }

    // A connection is considered secure if it goes to an HTTPs scheme or if it's the
    // localhost (which cannot be spoofed).
    fn is_secure(&self) -> bool {
        self.url.scheme() == "https" || self.url.host_str().unwrap_or("") == "localhost"
    }

    #[context("Failed to read http auth map.")]
    fn read_http_auth_map(&self) -> DfxResult<BTreeMap<String, String>> {
        let p = &Self::http_auth_path()?;
        let content = std::fs::read_to_string(p)
            .with_context(|| format!("Failed to read {}.", p.to_string_lossy()))?;

        // If there's an error parsing, simply use an empty map.
        Ok(
            serde_json::from_slice::<BTreeMap<String, String>>(content.as_bytes())
                .unwrap_or_else(|_| BTreeMap::new()),
        )
    }

    fn read_http_auth(&self) -> DfxResult<Option<(String, String)>> {
        match self.url.host() {
            None => Ok(None),
            Some(h) => {
                let map = self.read_http_auth_map()?;
                if let Some(token) = map.get(&h.to_string()) {
                    if !self.is_secure() {
                        slog::warn!(
                        self.logger,
                        "HTTP Auth was found, but protocol is not secure. Refusing to use the token."
                    );
                        Ok(None)
                    } else {
                        // For backward compatibility with previous versions of DFX, we still
                        // store the base64 encoding of `username:password`, but we decode it
                        // since the Agent requires username and password as separate fields.
                        let pair = base64::decode(&token).unwrap();
                        let pair = String::from_utf8_lossy(pair.as_slice());
                        let colon_pos = pair
                            .find(':')
                            .ok_or_else(|| anyhow!("Incorrectly formatted auth string (no `:`)"))?;
                        Ok(Some((
                            pair[..colon_pos].to_owned(),
                            pair[colon_pos + 1..].to_owned(),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn save_http_auth(&self, host: &str, auth: &str) -> DfxResult<PathBuf> {
        let mut map = self
            .read_http_auth_map()
            .unwrap_or_else(|_| BTreeMap::new());
        map.insert(host.to_string(), auth.to_string());

        let p = Self::http_auth_path()?;
        std::fs::write(
            &p,
            serde_json::to_string(&map)
                .context("Failed to serialize http auth map.")?
                .as_bytes(),
        )
        .with_context(|| format!("Failed to write to {}.", p.to_string_lossy()))?;

        Ok(p)
    }
}

impl ic_agent::agent::http_transport::PasswordManager for AgentClient {
    fn cached(&self, _url: &str) -> Result<Option<(String, String)>, String> {
        // Support for HTTP Auth if necessary (tries to contact first, then do the HTTP Auth
        // flow).
        if let Some(auth) = self.auth.lock().unwrap().as_ref() {
            Ok(Some(auth.clone()))
        } else {
            Ok(None)
        }
    }

    fn required(&self, _url: &str) -> Result<(String, String), String> {
        eprintln!("Unauthorized HTTP Access... Please enter credentials:");
        let mut username;
        while {
            username = dialoguer::Input::<String>::new()
                .with_prompt("Username")
                .interact()
                .unwrap();
            username.contains(':')
        } {
            eprintln!("Invalid username (unexpected `:`)")
        }
        let password = dialoguer::Password::new()
            .with_prompt("Password")
            .interact()
            .unwrap();

        let auth = format!("{}:{}", username, password);
        let auth = base64::encode(&auth);

        self.auth
            .lock()
            .unwrap()
            .replace((username.clone(), password.clone()));

        if let Some(h) = &self.url.host() {
            if let Ok(p) = self.save_http_auth(&h.to_string(), &auth) {
                slog::info!(
                    self.logger,
                    "Saved HTTP credentials to {}.",
                    p.to_string_lossy()
                );
            }
        }

        Ok((username, password))
    }
}

#[context("Failed to create agent with url {}.", url)]
pub fn create_agent(
    logger: Logger,
    url: &str,
    identity: Box<dyn Identity + Send + Sync>,
    timeout: Duration,
) -> DfxResult<Agent> {
    let executor = AgentClient::new(logger, url.to_string())?;
    let agent = Agent::builder()
        .with_transport(
            ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport::create(url)?
                .with_password_manager(executor),
        )
        .with_boxed_identity(identity)
        .with_ingress_expiry(Some(timeout))
        .build()?;
    Ok(agent)
}

#[cfg(test)]
mod tests {
    use std::{env, io};

    use slog::{o, Drain, Logger};
    use slog_term::{FullFormat, PlainSyncDecorator};
    use tempfile::TempDir;

    use super::AgentClient;

    #[test]
    fn test_passwords() {
        let cache_root = TempDir::new().unwrap();
        let old_var = env::var_os("DFX_CACHE_ROOT");
        env::set_var("DFX_CACHE_ROOT", cache_root.path());
        let log = Logger::root(
            FullFormat::new(PlainSyncDecorator::new(io::stderr()))
                .build()
                .fuse(),
            o!(),
        );
        let client = AgentClient::new(log, "https://localhost".to_owned()).unwrap();
        client
            .save_http_auth("localhost", &base64::encode("default:hunter2:"))
            .unwrap();
        let (user, pass) = client.read_http_auth().unwrap().unwrap();
        assert_eq!(user, "default");
        assert_eq!(pass, "hunter2:");
        if let Some(old_var) = old_var {
            env::set_var("DFX_CACHE_ROOT", old_var);
        } else {
            env::remove_var("DFX_CACHE_ROOT");
        }
    }
}
