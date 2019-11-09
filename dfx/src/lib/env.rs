use crate::config::dfinity::Config;
use crate::config::{cache, dfx_version};
use crate::lib::api_client::{Client, ClientConfig};
use crate::lib::error::DfxResult;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// An environment that contains the platform and general environment.
pub trait PlatformEnv {
    fn get_current_dir(&self) -> PathBuf;
}

/// An environment that manages the global binary cache.
pub trait BinaryCacheEnv {
    fn is_installed(&self) -> DfxResult<bool>;
    fn install(&self) -> DfxResult<()>;
}

/// An environment that can resolve binaries from the user-level cache.
pub trait BinaryResolverEnv {
    fn get_binary_command_path(&self, binary_name: &str) -> DfxResult<PathBuf>;
    fn get_binary_command(&self, binary_name: &str) -> DfxResult<std::process::Command>;
}

/// An environment that can get the project configuration.
pub trait ProjectConfigEnv {
    fn is_in_project(&self) -> bool;
    fn get_config(&self) -> Option<&Config>;
    fn get_dfx_root(&self) -> Option<&Path>;
}

/// An environment that can create clients from environment.
pub trait ClientEnv {
    fn get_client(&self) -> Client;
}

/// An environment that can get the version of the DFX we should be using.
pub trait VersionEnv {
    fn get_version(&self) -> &String;
}

/// An environment that is inside a project.
pub struct InProjectEnvironment {
    version: String,
    config: Config,
    temp_dir: PathBuf,
    client: RefCell<Option<Client>>,
}

impl PlatformEnv for InProjectEnvironment {
    fn get_current_dir(&self) -> PathBuf {
        let config_path = self.config.get_path();
        PathBuf::from(config_path.parent().unwrap())
    }
}

impl BinaryCacheEnv for InProjectEnvironment {
    fn is_installed(&self) -> DfxResult<bool> {
        cache::is_version_installed(self.version.as_str())
    }
    fn install(&self) -> DfxResult<()> {
        cache::install_version(self.version.as_str()).map(|_| ())
    }
}

impl BinaryResolverEnv for InProjectEnvironment {
    fn get_binary_command_path(&self, binary_name: &str) -> DfxResult<PathBuf> {
        cache::get_binary_path_from_version(self.version.as_str(), binary_name)
    }
    fn get_binary_command(&self, binary_name: &str) -> DfxResult<std::process::Command> {
        cache::binary_command_from_version(self.version.as_str(), binary_name)
    }
}

impl ProjectConfigEnv for InProjectEnvironment {
    fn is_in_project(&self) -> bool {
        true
    }
    fn get_config(&self) -> Option<&Config> {
        Some(&self.config)
    }
    fn get_dfx_root(&self) -> Option<&Path> {
        Some(&self.temp_dir)
    }
}

impl ClientEnv for InProjectEnvironment {
    fn get_client(&self) -> Client {
        {
            let mut cache = self.client.borrow_mut();
            if cache.is_some() {
                return cache.as_ref().unwrap().clone();
            }

            let start = self.config.get_config().get_defaults().get_start();
            let address = start.get_address("localhost");
            let port = start.get_port(8080);

            *cache = Some(Client::new(ClientConfig {
                url: format!("http://{}:{}", address, port),
            }));
        }

        // Have to recursively call ourselves to avoid cache getting out of scope.
        self.get_client()
    }
}

impl VersionEnv for InProjectEnvironment {
    fn get_version(&self) -> &String {
        &self.version
    }
}

impl InProjectEnvironment {
    pub fn from_current_dir() -> DfxResult<InProjectEnvironment> {
        let config = Config::from_current_dir()?;
        let temp_dir = config.get_path().parent().unwrap().join(".dfx");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(InProjectEnvironment {
            version: config
                .get_config()
                .get_dfx()
                .unwrap_or_else(|| dfx_version().to_owned()),
            config,
            temp_dir,
            client: RefCell::new(None),
        })
    }
}

pub struct GlobalEnvironment {
    version: String,
}

impl PlatformEnv for GlobalEnvironment {
    fn get_current_dir(&self) -> PathBuf {
        std::env::current_dir().unwrap()
    }
}

impl BinaryCacheEnv for GlobalEnvironment {
    fn is_installed(&self) -> DfxResult<bool> {
        cache::is_version_installed(self.version.as_str())
    }
    fn install(&self) -> DfxResult<()> {
        cache::install_version(self.version.as_str()).map(|_| ())
    }
}

impl BinaryResolverEnv for GlobalEnvironment {
    fn get_binary_command_path(&self, binary_name: &str) -> DfxResult<PathBuf> {
        cache::get_binary_path_from_version(self.version.as_str(), binary_name)
    }
    fn get_binary_command(&self, binary_name: &str) -> DfxResult<std::process::Command> {
        cache::binary_command_from_version(self.version.as_str(), binary_name)
    }
}

impl ProjectConfigEnv for GlobalEnvironment {
    fn is_in_project(&self) -> bool {
        false
    }
    fn get_config(&self) -> Option<&Config> {
        None
    }
    fn get_dfx_root(&self) -> Option<&Path> {
        None
    }
}

impl ClientEnv for GlobalEnvironment {
    fn get_client(&self) -> Client {
        panic!();
    }
}

impl VersionEnv for GlobalEnvironment {
    fn get_version(&self) -> &String {
        &self.version
    }
}

impl GlobalEnvironment {
    pub fn from_current_dir() -> DfxResult<GlobalEnvironment> {
        Ok(GlobalEnvironment {
            version: dfx_version().to_owned(),
        })
    }
}
