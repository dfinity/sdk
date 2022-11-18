use crate::lib::error::DfxResult;
#[cfg(windows)]
use crate::util::project_dirs;

use anyhow::{bail, Context};
use fn_error_context::context;
use std::path::PathBuf;

#[context("Failed to get path to dfx config dir.")]
pub fn get_config_dfx_dir_path() -> DfxResult<PathBuf> {
    let config_root = std::env::var_os("DFX_CONFIG_ROOT");
    // dirs-next is not used for *nix to preserve existing paths
    #[cfg(not(windows))]
    let p = {
        let home = std::env::var_os("HOME").context("Failed to resolve 'HOME' env var.")?;
        let root = config_root.unwrap_or(home);
        PathBuf::from(root).join(".config").join("dfx")
    };
    #[cfg(windows)]
    let p = match config_root {
        Some(var) => PathBuf::from(var),
        None => project_dirs()?.config_dir().to_owned(),
    };
    if !p.exists() {
        std::fs::create_dir_all(&p)
            .with_context(|| format!("Cannot create config directory at {}", p.display()))?;
    } else if !p.is_dir() {
        bail!("Path {} is not a directory.", p.display());
    }
    Ok(p)
}
