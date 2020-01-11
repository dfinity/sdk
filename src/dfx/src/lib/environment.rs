use crate::config::cache::{Cache, DiskBasedCache};
use crate::config::dfinity::Config;
use crate::lib::api_client::{Client, ClientConfig};
use crate::lib::error::DfxResult;
use semver::Version;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Environment {
    fn get_cache(&self) -> Rc<dyn Cache>;
    fn get_config(&self) -> Option<Rc<Config>>;

    fn is_in_project(&self) -> bool;
    fn get_temp_dir(&self) -> &Path;
    fn get_version(&self) -> &Version;
    fn get_client(&self) -> Option<Client>;
}

pub struct EnvironmentImpl {
    config: Option<Rc<Config>>,
    temp_dir: PathBuf,

    client: RefCell<Option<Client>>,
    cache: Rc<dyn Cache>,

    version: Version,
}

impl EnvironmentImpl {
    pub fn new() -> DfxResult<Self> {
        use crate::config::dfx_version;

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

        let version = match &config {
            None => dfx_version().clone(),
            Some(c) => match &c.get_config().get_dfx() {
                None => dfx_version().clone(),
                Some(v) => Version::parse(&v)?,
            },
        };

        Ok(EnvironmentImpl {
            cache: Rc::new(DiskBasedCache::with_version(&version)),
            config: config.map(Rc::new),
            temp_dir,
            client: RefCell::new(None),
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

    fn get_client(&self) -> Option<Client> {
        {
            let mut cache = self.client.borrow_mut();
            if cache.is_some() {
                return Some(cache.as_ref().unwrap().clone());
            }

            let config = self
                .config
                .as_ref()
                .expect("Trying to access a client outside of a dfx project.");
            let start = config.get_config().get_defaults().get_start();
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
