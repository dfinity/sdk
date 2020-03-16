use crate::config::cache::{Cache, DiskBasedCache};
use crate::config::dfinity::Config;
use crate::config::dfx_version;
use crate::lib::error::DfxResult;
use crate::lib::identity_interface::Identity;
use crate::lib::progress_bar::ProgressBar;

use ic_http_agent::{Agent, AgentConfig};
use lazy_init::Lazy;
use semver::Version;
use slog::Record;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Environment {
    fn get_cache(&self) -> Rc<dyn Cache>;
    fn get_config(&self) -> Option<Rc<Config>>;

    fn is_in_project(&self) -> bool;
    /// Return a temporary directory for configuration if none exists
    /// for the current project or if not in a project. Following
    /// invocations by other processes in the same project should
    /// return the same configuration directory.
    fn get_temp_dir(&self) -> &Path;
    /// Return the directory where state for replica(s) is kept.
    fn get_state_dir(&self) -> PathBuf;
    fn get_version(&self) -> &Version;

    // Explicit lifetimes are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn get_agent<'a>(&'a self) -> Option<&'a Agent>;

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
    config: Option<Rc<Config>>,
    temp_dir: PathBuf,

    agent: Lazy<Option<Agent>>,
    cache: Rc<dyn Cache>,

    version: Version,

    logger: Option<slog::Logger>,
    progress: bool,
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
            cache: Rc::new(DiskBasedCache::with_version(&version)),
            config: config.map(Rc::new),
            temp_dir,
            agent: Lazy::new(),
            version: version.clone(),
            logger: None,
            progress: true,
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
}

impl Environment for EnvironmentImpl {
    fn get_cache(&self) -> Rc<dyn Cache> {
        Rc::clone(&self.cache)
    }

    fn get_config(&self) -> Option<Rc<Config>> {
        self.config.as_ref().map(|x| Rc::clone(x))
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

    fn get_agent(&self) -> Option<&Agent> {
        self.agent
            .get_or_create(move || {
                if let Some(config) = self.config.as_ref() {
                    let start = config.get_config().get_defaults().get_start();
                    let address = start.get_address("localhost");
                    let port = start.get_port(8000);
                    let dfx_root = self.get_temp_dir();
                    // This is the default to keep precedence sane.
                    let local_project_identity = dfx_root.join("identity").join("default");
                    if create_dir_all(&local_project_identity).is_err() {
                        return None;
                    }

                    Agent::new(AgentConfig {
                        url: format!("http://{}:{}", address, port).as_str(),
                        signer: Box::new(Identity::new(local_project_identity)),
                        ..AgentConfig::default()
                    })
                    .ok()
                } else {
                    None
                }
            })
            .as_ref()
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
}

impl<'a> AgentEnvironment<'a> {
    pub fn new(backend: &'a dyn Environment, agent_url: &str) -> Self {
        // We do not expose the path directly for now.
        let dfx_root = backend.get_temp_dir();
        // This is the default to keep precedence sane,
        // not deal with home folders or cache right now.
        let local_project_identity = dfx_root.join("identity").join("default");
        // This is for sanity. The environment should have created
        // this already. N.B. Do not assume the existence of this
        // directory yet.
        create_dir_all(&local_project_identity).expect("Failed to construct identity profile");
        AgentEnvironment {
            backend,
            agent: Agent::new(AgentConfig {
                url: agent_url,
                signer: Box::new(Identity::new(local_project_identity)),
                ..AgentConfig::default()
            })
            .expect("Failed to construct agent"),
        }
    }
}

impl<'a> Environment for AgentEnvironment<'a> {
    fn get_cache(&self) -> Rc<dyn Cache> {
        self.backend.get_cache()
    }

    fn get_config(&self) -> Option<Rc<Config>> {
        self.backend.get_config()
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

    fn get_agent(&self) -> Option<&Agent> {
        Some(&self.agent)
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
