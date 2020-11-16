use crate::config::cache::{Cache, DiskBasedCache};
use crate::config::dfinity::Config;
use crate::config::{cache, dfx_version};
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::network::network_descriptor::NetworkDescriptor;
use crate::lib::progress_bar::ProgressBar;

use anyhow::Context;
use ic_agent::{Agent, Identity};
use semver::Version;
use slog::{Logger, Record};
use std::collections::BTreeMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub trait Environment {
    fn get_cache(&self) -> Arc<dyn Cache>;
    fn get_config(&self) -> Option<Arc<Config>>;
    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>>;

    fn is_in_project(&self) -> bool;
    /// Return a temporary directory for configuration if none exists
    /// for the current project or if not in a project. Following
    /// invocations by other processes in the same project should
    /// return the same configuration directory.
    fn get_temp_dir(&self) -> &Path;
    /// Return the directory where state for replica(s) is kept.
    fn get_state_dir(&self) -> PathBuf;
    fn get_version(&self) -> &Version;

    /// This is value of the name passed to dfx `--identity <name>`
    /// Notably, it is _not_ the name of the default identity or selected identity
    fn get_identity_override(&self) -> &Option<String>;

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn get_agent<'a>(&'a self) -> Option<&'a Agent>;

    #[allow(clippy::needless_lifetimes)]
    fn get_network_descriptor<'a>(&'a self) -> Option<&'a NetworkDescriptor>;

    fn get_logger(&self) -> &slog::Logger;
    fn new_spinner(&self, message: &str) -> ProgressBar;
    fn new_progress(&self, message: &str) -> ProgressBar;

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn log<'a>(&self, record: &Record<'a>) {
        self.get_logger().log(record);
    }
}

pub struct EnvironmentImpl {
    config: Option<Arc<Config>>,
    temp_dir: PathBuf,

    cache: Arc<dyn Cache>,

    version: Version,

    logger: Option<slog::Logger>,
    progress: bool,

    identity_override: Option<String>,
}

impl EnvironmentImpl {
    pub fn new() -> DfxResult<Self> {
        let config = match Config::from_current_dir() {
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
            Ok(x) => Ok(Some(x)),
        }?;
        let temp_dir = match &config {
            None => tempfile::tempdir()
                .expect("Could not create a temporary directory.")
                .into_path(),
            Some(c) => c.get_path().parent().unwrap().join(".dfx"),
        };
        create_dir_all(&temp_dir)?;

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
                    Some(v) => Version::parse(&v)?,
                },
            },
            Ok(v) => {
                if v.is_empty() {
                    dfx_version().clone()
                } else {
                    Version::parse(&v)?
                }
            }
        };

        Ok(EnvironmentImpl {
            cache: Arc::new(DiskBasedCache::with_version(&version)),
            config: config.map(Arc::new),
            temp_dir,
            version: version.clone(),
            logger: None,
            progress: true,
            identity_override: None,
        })
    }

    pub fn with_logger(mut self, logger: slog::Logger) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn with_progress_bar(mut self, progress: bool) -> Self {
        self.progress = progress;
        self
    }

    pub fn with_identity_override(mut self, identity: Option<String>) -> Self {
        self.identity_override = identity;
        self
    }
}

impl Environment for EnvironmentImpl {
    fn get_cache(&self) -> Arc<dyn Cache> {
        Arc::clone(&self.cache)
    }

    fn get_config(&self) -> Option<Arc<Config>> {
        self.config.as_ref().map(|x| Arc::clone(x))
    }

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config().ok_or(anyhow::anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn is_in_project(&self) -> bool {
        self.config.is_some()
    }

    fn get_temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    fn get_state_dir(&self) -> PathBuf {
        self.get_temp_dir().join("state")
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

    fn get_network_descriptor(&self) -> Option<&NetworkDescriptor> {
        // create an AgentEnvironment explicitly, in order to specify network and agent.
        // See install, build for examples.
        None
    }

    fn get_logger(&self) -> &slog::Logger {
        self.logger
            .as_ref()
            .expect("Log was not setup, but is being used.")
    }

    fn new_spinner(&self, message: &str) -> ProgressBar {
        if self.progress {
            ProgressBar::new_spinner(message)
        } else {
            ProgressBar::discard()
        }
    }

    fn new_progress(&self, _message: &str) -> ProgressBar {
        ProgressBar::discard()
    }
}

pub struct AgentEnvironment<'a> {
    backend: &'a dyn Environment,
    agent: Agent,
    network_descriptor: NetworkDescriptor,
}

impl<'a> AgentEnvironment<'a> {
    pub fn new(
        backend: &'a dyn Environment,
        network_descriptor: NetworkDescriptor,
        timeout: Duration,
    ) -> DfxResult<Self> {
        let identity = IdentityManager::new(backend)?.instantiate_selected_identity()?;

        let agent_url = network_descriptor.providers.first().unwrap();
        Ok(AgentEnvironment {
            backend,
            agent: create_agent(backend.get_logger().clone(), agent_url, identity, timeout)
                .expect("Failed to construct agent."),
            network_descriptor,
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

    fn get_config_or_anyhow(&self) -> anyhow::Result<Arc<Config>> {
        self.get_config().ok_or(anyhow::anyhow!(
            "Cannot find dfx configuration file in the current working directory. Did you forget to create one?"
        ))
    }

    fn is_in_project(&self) -> bool {
        self.backend.is_in_project()
    }

    fn get_temp_dir(&self) -> &Path {
        self.backend.get_temp_dir()
    }

    fn get_state_dir(&self) -> PathBuf {
        self.backend.get_state_dir()
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

    fn get_network_descriptor(&self) -> Option<&NetworkDescriptor> {
        Some(&self.network_descriptor)
    }

    fn get_logger(&self) -> &slog::Logger {
        self.backend.get_logger()
    }

    fn new_spinner(&self, message: &str) -> ProgressBar {
        self.backend.new_spinner(message)
    }

    fn new_progress(&self, message: &str) -> ProgressBar {
        self.backend.new_progress(message)
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
        let url = reqwest::Url::parse(&url).context("Invalid URL: {}", url)?;

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

    fn http_auth_path() -> DfxResult<PathBuf> {
        Ok(cache::get_cache_root()?.join("http_auth"))
    }

    // A connection is considered secure if it goes to an HTTPs scheme or if it's the
    // localhost (which cannot be spoofed).
    fn is_secure(&self) -> bool {
        self.url.scheme() == "https" || self.url.host_str().unwrap_or("") == "localhost"
    }

    fn read_http_auth_map(&self) -> DfxResult<BTreeMap<String, String>> {
        let p = &Self::http_auth_path()?;
        let content = std::fs::read_to_string(p)?;

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
                        let v: Vec<String> = String::from_utf8_lossy(pair.as_slice())
                            .split(':')
                            .take(2)
                            .map(|s| s.to_owned())
                            .collect();
                        Ok(Some((v[0].to_owned(), v[1].to_owned())))
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
        std::fs::write(&p, serde_json::to_string(&map)?.as_bytes())?;

        Ok(p)
    }
}

impl ic_agent::PasswordManager for AgentClient {
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
        let username = dialoguer::Input::<String>::new()
            .with_prompt("Username")
            .interact()
            .unwrap();
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

fn create_agent<I>(logger: Logger, url: &str, identity: Box<I>, timeout: Duration) -> Option<Agent>
where
    I: Identity + Send + Sync + 'static,
{
    AgentClient::new(logger, url.to_string())
        .ok()
        .and_then(|executor| {
            Agent::builder()
                .with_url(url)
                .with_boxed_identity(identity)
                .with_password_manager(executor)
                .with_ingress_expiry(Some(timeout))
                .build()
                .ok()
        })
}
