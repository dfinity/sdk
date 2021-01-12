use crate::lib::error::DfxResult;

use anyhow::{bail, Context};
use std::path::PathBuf;

pub fn get_config_dfx_dir_path() -> DfxResult<PathBuf> {
    let config_root = std::env::var("DFX_CONFIG_ROOT").ok();
    let home = std::env::var("HOME").context("Cannot find home directroy.")?;
    let root = config_root.unwrap_or(home);
    let p = PathBuf::from(root).join(".config").join("dfx");
    if !p.exists() {
        std::fs::create_dir_all(&p)
            .context(format!("Cannot create config directory at {}", p.display()))?;
    } else if !p.is_dir() {
        bail!("Path {} is not a directory.", p.display());
    }
    Ok(p)
}
