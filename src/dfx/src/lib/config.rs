#[cfg(windows)]
use dfx_core::config::directories::project_dirs;
use dfx_core::error::config::ConfigError;
use dfx_core::error::config::ConfigError::{
    DetermineConfigDirectoryFailed, EnsureConfigDirectoryExistsFailed,
};
use dfx_core::foundation::get_user_home;
use dfx_core::fs::composite::ensure_dir_exists;

use std::path::PathBuf;

pub fn get_config_dfx_dir_path() -> Result<PathBuf, ConfigError> {
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
