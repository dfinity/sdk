use crate::lib::error::DfxResult;

use anyhow::{bail, Context};
use fn_error_context::context;
use std::path::PathBuf;

#[context("Failed to get path to dfx config dir.")]
pub fn get_config_dfx_dir_path() -> DfxResult<PathBuf> {
    let config_root = std::env::var("DFX_CONFIG_ROOT").ok();
    let home = std::env::var("HOME").context("Failed to resolve 'HOME' env var.")?;
    let root = config_root.unwrap_or(home);
    let p = PathBuf::from(root).join(".config").join("dfx");
    if !p.exists() {
        std::fs::create_dir_all(&p)
            .with_context(|| format!("Cannot create config directory at {}", p.display()))?;
    } else if !p.is_dir() {
        bail!("Path {} is not a directory.", p.display());
    }
    Ok(p)
}
