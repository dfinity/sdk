use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult, IdentityErrorKind};
use ic_agent::{BasicIdentity, Identity};
use pem::{encode, Pem};
use ring::{rand, signature};
use serde::{Deserialize, Serialize};
use slog::Logger;
use std::fs;
use std::path::PathBuf;

const DEFAULT_IDENTITY_NAME: &str = "default";
const ANONYMOUS_IDENTITY_NAME: &str = "anonymous";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Configuration {
    #[serde(default = "default_identity")]
    pub default: String,
}

fn default_identity() -> String {
    String::from(DEFAULT_IDENTITY_NAME)
}

#[derive(Clone, Debug)]
pub struct IdentityManager {
    identity_json_path: PathBuf,
    identity_root_path: PathBuf,
    configuration: Configuration,
    selected_identity: String,
}

impl IdentityManager {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        let config_dfx_dir_path = get_config_dfx_dir_path()?;
        let identity_root_path = config_dfx_dir_path.join("identity");
        let identity_json_path = config_dfx_dir_path.join("identity.json");

        let configuration = if identity_json_path.exists() {
            IdentityManager::read_configuration(&identity_json_path)
        } else {
            IdentityManager::initialize(env.get_logger(), &identity_json_path, &identity_root_path)
        }?;

        let identity_override = env.get_identity_override();
        let selected_identity = identity_override
            .clone()
            .unwrap_or_else(|| configuration.default.clone());

        let mgr = IdentityManager {
            identity_json_path,
            identity_root_path,
            configuration,
            selected_identity,
        };

        if let Some(identity) = identity_override {
            mgr.require_identity_exists(&identity)?;
        }

        Ok(mgr)
    }

    pub fn instantiate_selected_identity(&self) -> DfxResult<Box<dyn Identity>> {
        let pem_path = self.get_selected_identity_pem_path();
        let basic = BasicIdentity::from_pem_file(&pem_path).map_err(|e| {
            DfxError::IdentityError(IdentityErrorKind::AgentPemError(e, pem_path.clone()))
        })?;

        let b: Box<dyn Identity> = Box::new(basic);
        Ok(b)
    }

    pub fn create_new_identity(&self, name: &str) -> DfxResult {
        if name == ANONYMOUS_IDENTITY_NAME {
            return Err(DfxError::IdentityError(
                IdentityErrorKind::CannotCreateAnonymousIdentity(),
            ));
        }
        let identity_dir = self.get_identity_dir_path(name);

        if identity_dir.exists() {
            return Err(DfxError::IdentityError(
                IdentityErrorKind::IdentityAlreadyExists(),
            ));
        }
        std::fs::create_dir_all(&identity_dir).map_err(|e| {
            DfxError::IdentityError(IdentityErrorKind::CouldNotCreateIdentityDirectory(
                identity_dir.clone(),
                e,
            ))
        })?;

        let pem_file = identity_dir.join("identity.pem");
        IdentityManager::generate_key(&pem_file)
    }

    pub fn get_identity_names(&self) -> DfxResult<Vec<String>> {
        let mut names = self
            .identity_root_path
            .read_dir()?
            .filter(|entry_result| match entry_result {
                Ok(dir_entry) => match dir_entry.file_type() {
                    Ok(file_type) => file_type.is_dir(),
                    _ => false,
                },
                _ => false,
            })
            .map(|entry_result| {
                entry_result.map(|entry| entry.file_name().to_string_lossy().to_string())
            })
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        names.sort();

        Ok(names)
    }

    pub fn get_selected_identity_name(&self) -> &String {
        &self.selected_identity
    }

    pub fn remove(&self, name: &str) -> DfxResult {
        self.require_identity_exists(name)?;

        if self.configuration.default == name {
            return Err(DfxError::IdentityError(
                IdentityErrorKind::CannotDeleteDefaultIdentity(),
            ));
        }
        let dir = self.get_identity_dir_path(name);
        let pem = self.get_identity_pem_path(name);

        std::fs::remove_file(&pem).map_err(|e| DfxError::IoWithPath(e, pem))?;
        std::fs::remove_dir(&dir).map_err(|e| DfxError::IoWithPath(e, dir))
    }

    pub fn rename(&self, from: &str, to: &str) -> DfxResult<bool> {
        if to == ANONYMOUS_IDENTITY_NAME {
            return Err(DfxError::IdentityError(
                IdentityErrorKind::CannotCreateAnonymousIdentity(),
            ));
        }
        self.require_identity_exists(from)?;

        let from_dir = self.get_identity_dir_path(from);
        let to_dir = self.get_identity_dir_path(to);

        if to_dir.exists() {
            return Err(DfxError::IdentityError(
                IdentityErrorKind::IdentityAlreadyExists(),
            ));
        }

        std::fs::rename(&from_dir, &to_dir).map_err(|e| {
            DfxError::IdentityError(IdentityErrorKind::CouldNotRenameIdentityDirectory(
                from_dir, to_dir, e,
            ))
        })?;

        if from == self.configuration.default {
            self.write_default_identity(to)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn r#use(&self, name: &str) -> DfxResult {
        self.require_identity_exists(name)?;
        self.write_default_identity(name)
    }

    fn initialize(
        logger: &Logger,
        identity_json_path: &PathBuf,
        identity_root_path: &PathBuf,
    ) -> DfxResult<Configuration> {
        let identity_dir = identity_root_path.join(DEFAULT_IDENTITY_NAME);
        let identity_pem_path = identity_dir.join("identity.pem");
        if !identity_pem_path.exists() {
            if !identity_dir.exists() {
                std::fs::create_dir_all(&identity_dir).map_err(|e| {
                    DfxError::IdentityError(IdentityErrorKind::CouldNotCreateIdentityDirectory(
                        identity_dir.clone(),
                        e,
                    ))
                })?;
            }

            let creds_pem_path = IdentityManager::get_legacy_creds_pem_path()?;
            if creds_pem_path.exists() {
                fs::copy(creds_pem_path, identity_pem_path)?;
            } else {
                IdentityManager::generate_key(&identity_pem_path)?;
            }
        }

        let config = Configuration {
            default: String::from(DEFAULT_IDENTITY_NAME),
        };
        IdentityManager::write_configuration(&identity_json_path, &config)?;

        slog::info!(logger, r#"Created the "default" identity."#);
        Ok(config)
    }

    fn write_default_identity(&self, name: &str) -> DfxResult {
        let config = Configuration {
            default: String::from(name),
        };
        IdentityManager::write_configuration(&self.identity_json_path, &config)
    }

    fn require_identity_exists(&self, name: &str) -> DfxResult {
        let identity_pem_path = self.get_identity_pem_path(name);

        if !identity_pem_path.exists() {
            Err(DfxError::IdentityError(
                IdentityErrorKind::IdentityDoesNotExist(String::from(name), identity_pem_path),
            ))
        } else {
            Ok(())
        }
    }

    fn get_identity_dir_path(&self, identity: &str) -> PathBuf {
        self.identity_root_path.join(&identity)
    }

    fn get_identity_pem_path(&self, identity: &str) -> PathBuf {
        self.get_identity_dir_path(identity).join("identity.pem")
    }

    fn get_selected_identity_pem_path(&self) -> PathBuf {
        self.get_identity_pem_path(&self.selected_identity)
    }

    fn get_legacy_creds_pem_path() -> DfxResult<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| {
            DfxError::IdentityError(IdentityErrorKind::CannotFindUserHomeDirectory())
        })?;

        Ok(PathBuf::from(home)
            .join(".dfinity")
            .join("identity")
            .join("creds.pem"))
    }

    fn read_configuration(path: &PathBuf) -> DfxResult<Configuration> {
        let content =
            std::fs::read_to_string(&path).map_err(|e| DfxError::IoWithPath(e, path.clone()))?;
        serde_json::from_str(&content).map_err(DfxError::from)
    }

    fn write_configuration(path: &PathBuf, config: &Configuration) -> DfxResult {
        let content = serde_json::to_string_pretty(&config)?;

        std::fs::write(&path, content).map_err(|err| DfxError::IoWithPath(err, path.clone()))
    }

    fn generate_key(pem_file: &PathBuf) -> DfxResult {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|x| DfxError::IdentityError(IdentityErrorKind::CouldNotGenerateKey(x)))?;

        let encoded_pem = IdentityManager::encode_pem_private_key(&(*pkcs8_bytes.as_ref()));
        fs::write(&pem_file, encoded_pem)?;

        let mut permissions = fs::metadata(&pem_file)?.permissions();
        permissions.set_readonly(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(0o400);
        }

        fs::set_permissions(&pem_file, permissions)?;

        Ok(())
    }

    fn encode_pem_private_key(key: &[u8]) -> String {
        let pem = Pem {
            tag: "PRIVATE KEY".to_owned(),
            contents: key.to_vec(),
        };
        encode(&pem)
    }
}
