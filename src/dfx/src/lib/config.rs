use crate::lib::error::{ConfigErrorKind, DfxError, DfxResult};
use std::path::PathBuf;

pub fn get_config_dfx_dir_path() -> DfxResult<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| DfxError::ConfigError(ConfigErrorKind::CannotFindUserHomeDirectory()))?;

    let p = PathBuf::from(home).join(".config").join("dfx");

    if !p.exists() {
        std::fs::create_dir_all(&p).map_err(|e| {
            DfxError::ConfigError(ConfigErrorKind::CouldNotCreateConfigDirectory(p.clone(), e))
        })?;
    } else if !p.is_dir() {
        return Err(DfxError::ConfigError(
            ConfigErrorKind::HomeConfigDfxShouldBeADirectory(p),
        ));
    }

    Ok(p)
}
