use crate::lib::error::DfxResult;

use anyhow::{bail, Context};
use std::path::PathBuf;

pub fn get_config_dfx_dir_path() -> DfxResult<PathBuf> {
    let home = std::env::var("HOME").context("Cannot find home directroy.")?;
    let path = PathBuf::from(home).join(".config").join("dfx");
    if !path.exists() {
        std::fs::create_dir_all(&path).context(format!(
            "Cannot create config directory at {}",
            path.display()
        ))?;
    } else if !path.is_dir() {
        bail!("Path {} is not a directory.", path.display());
    }
    Ok(path)
}
