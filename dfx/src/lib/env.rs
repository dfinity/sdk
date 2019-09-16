use crate::config::dfinity::Config;
use crate::config::{cache, DFX_VERSION};
use crate::lib::api_client::{Client, ClientConfig};
use crate::lib::error::DfxResult;
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;

/// An environment that contains the platform and general environment.
pub trait PlatformEnv {
    fn get_current_dir(&self) -> PathBuf;
}

/// An environment that manages the global binary cache.
pub trait BinaryCacheEnv {
    fn is_installed(&self) -> io::Result<bool>;
    fn install(&self) -> io::Result<()>;
}

/// An environment that can resolve binaries from the user-level cache.
pub trait BinaryResolverEnv {
    fn get_binary_command_path(&self, binary_name: &str) -> io::Result<PathBuf>;
    fn get_binary_command(&self, binary_name: &str) -> io::Result<std::process::Command>;
}

/// An environment that can get the project configuration.
pub trait ProjectConfigEnv {
    fn is_in_project(&self) -> bool;
    fn get_config(&self) -> Option<&Config>;
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
    client: RefCell<Option<Client>>,
}

impl PlatformEnv for InProjectEnvironment {
    fn get_current_dir(&self) -> PathBuf {
        let config_path = self.get_config().unwrap().get_path();
        PathBuf::from(config_path.parent().unwrap())
    }
}

impl BinaryCacheEnv for InProjectEnvironment {
    fn is_installed(&self) -> io::Result<bool> {
        cache::is_version_installed(self.version.as_str())
    }
    fn install(&self) -> io::Result<()> {
        cache::install_version(self.version.as_str()).map(|_| ())
    }
}

impl BinaryResolverEnv for InProjectEnvironment {
    fn get_binary_command_path(&self, binary_name: &str) -> io::Result<PathBuf> {
        cache::get_binary_path_from_version(self.version.as_str(), binary_name)
    }
    fn get_binary_command(&self, binary_name: &str) -> io::Result<std::process::Command> {
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

        Ok(InProjectEnvironment {
            version: config
                .get_config()
                .get_dfx()
                .unwrap_or_else(|| DFX_VERSION.to_owned()),
            config,
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
    fn is_installed(&self) -> io::Result<bool> {
        cache::is_version_installed(self.version.as_str())
    }
    fn install(&self) -> io::Result<()> {
        cache::install_version(self.version.as_str()).map(|_| ())
    }
}

impl BinaryResolverEnv for GlobalEnvironment {
    fn get_binary_command_path(&self, binary_name: &str) -> std::io::Result<PathBuf> {
        cache::get_binary_path_from_version(self.version.as_str(), binary_name)
    }
    fn get_binary_command(&self, binary_name: &str) -> std::io::Result<std::process::Command> {
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
        #[cfg(debug_assertions)]
        {
            // In debug, add a timestamp of the compilation at the end of version to ensure all
            // debug runs are unique (and cached uniquely).
            Ok(GlobalEnvironment {
                version: format!(
                    "{}-{:?}",
                    DFX_VERSION.to_owned(),
                    std::env::var_os("DFX_TIMESTAMP_DEBUG_MODE_ONLY")
                        .unwrap_or_else(|| std::ffi::OsString::from("local-debug"))
                ),
            })
        }

        #[cfg(not(debug_assertions))]
        {
            Ok(GlobalEnvironment {
                version: DFX_VERSION.to_owned(),
            })
        }
    }
}
