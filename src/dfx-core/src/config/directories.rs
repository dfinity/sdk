use crate::config::model::local_server_descriptor::NetworkMetadata;
use crate::error::config::ConfigError::{
    DetermineConfigDirectoryFailed, EnsureConfigDirectoryExistsFailed,
};
use crate::error::config::{ConfigError, GetSharedWalletConfigPathError};
use crate::error::get_user_home::GetUserHomeError;
use crate::error::get_user_home::GetUserHomeError::NoHomeInEnvironment;
#[cfg(not(windows))]
use crate::foundation::get_user_home;
use crate::fs::composite::ensure_dir_exists;
use crate::identity::WALLET_CONFIG_FILENAME;
use crate::json::load_json_file;
use directories_next::ProjectDirs;
use std::path::PathBuf;

pub fn project_dirs() -> Result<&'static ProjectDirs, GetUserHomeError> {
    lazy_static::lazy_static! {
        static ref DIRS: Option<ProjectDirs> = ProjectDirs::from("org", "dfinity", "dfx");
    }
    DIRS.as_ref().ok_or(NoHomeInEnvironment())
}

pub fn get_shared_network_data_directory(network: &str) -> Result<PathBuf, GetUserHomeError> {
    Ok(project_dirs()?
        .data_local_dir()
        .join("network")
        .join(network))
}

pub fn get_shared_wallet_config_path(
    network: &str,
) -> Result<Option<PathBuf>, GetSharedWalletConfigPathError> {
    let data_dir = get_shared_network_data_directory(network)?;

    let network_id_path = data_dir.join("network-id");
    if network_id_path.exists() {
        let network_metadata: NetworkMetadata = load_json_file(&network_id_path)?;
        let path = data_dir
            .join(network_metadata.settings_digest)
            .join(WALLET_CONFIG_FILENAME);
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

pub fn get_user_dfx_config_dir() -> Result<PathBuf, ConfigError> {
    let config_root = std::env::var_os("DFX_CONFIG_ROOT");
    // dirs-next is not used for *nix to preserve existing paths
    #[cfg(not(windows))]
    let p = {
        let home = get_user_home().map_err(DetermineConfigDirectoryFailed)?;
        let root = config_root.unwrap_or(home);
        PathBuf::from(root).join(".config").join("dfx")
    };
    #[cfg(windows)]
    let p = match config_root {
        Some(var) => PathBuf::from(var),
        None => project_dirs()
            .map_err(DetermineConfigDirectoryFailed)?
            .config_dir()
            .to_owned(),
    };
    ensure_dir_exists(&p).map_err(EnsureConfigDirectoryExistsFailed)?;
    Ok(p)
}
