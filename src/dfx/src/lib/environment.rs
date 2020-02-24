use crate::config::cache::{Cache, DiskBasedCache};
use crate::config::dfinity::Config;
use crate::config::dfx_version;
use crate::lib::error::DfxResult;
use ic_http_agent::{Agent, AgentConfig};
use lazy_init::Lazy;
use semver::Version;
use std::fs::read_to_string;
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
    fn get_version(&self) -> &Version;

    // Timelines are actually needed for mockall to work properly.
    #[allow(clippy::needless_lifetimes)]
    fn get_agent<'a>(&'a self) -> Option<&'a Agent>;
}

pub struct EnvironmentImpl {
    config: Option<Rc<Config>>,
    temp_dir: PathBuf,

    agent: Lazy<Option<Agent>>,
    cache: Rc<dyn Cache>,

    version: Version,
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
        std::fs::create_dir_all(&temp_dir)?;

        // Figure out which version of DFX we should be running. This will use the following
        // fallback sequence:
        //   1. DFX_VERSION environment variable
        //   2. dfx.json "dfx" field
        //   3. this binary's version
        let version = match std::env::var("DFX_VERSION") {
            Err(_) => match &config {
                None => dfx_version().clone(),
                Some(c) => match &c.get_config().get_dfx() {
                    None => dfx_version().clone(),
                    Some(v) => Version::parse(&v)?,
                },
            },
            Ok(v) => Version::parse(&v)?,
        };

        Ok(EnvironmentImpl {
            cache: Rc::new(DiskBasedCache::with_version(&version)),
            config: config.map(Rc::new),
            temp_dir,
            agent: Lazy::new(),
            version: version.clone(),
        })
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

    fn get_version(&self) -> &Version {
        &self.version
    }

    fn get_agent(&self) -> Option<&Agent> {
        self.agent
            .get_or_create(move || {
                if let Some(config) = self.config.as_ref() {
                    let start = config.get_config().get_defaults().get_start();
                    let address = start.get_address("localhost");
                    let dfx_root = self.get_temp_dir();
                    let client_configuration_dir = dfx_root.join("client-configuration");
                    let client_port_path = client_configuration_dir.join("client-1.port");
                    let port = read_to_string(&client_port_path)
                        .expect("Could not read port configuration file");

                    Agent::new(AgentConfig {
                        url: format!("http://{}:{}", address, port).as_str(),
                        ..AgentConfig::default()
                    })
                    .ok()
                } else {
                    None
                }
            })
            .as_ref()
    }
}

pub struct AgentEnvironment<'a> {
    backend: &'a dyn Environment,
    agent: Agent,
}

impl<'a> AgentEnvironment<'a> {
    pub fn new(backend: &'a dyn Environment, agent_url: &str) -> Self {
        AgentEnvironment {
            backend,
            agent: Agent::new(AgentConfig {
                url: agent_url,
                ..AgentConfig::default()
            })
            .unwrap(),
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

    fn get_version(&self) -> &Version {
        self.backend.get_version()
    }

    fn get_agent(&self) -> Option<&Agent> {
        Some(&self.agent)
    }
}
