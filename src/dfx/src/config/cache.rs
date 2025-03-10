use crate::config::dfx_version;
use crate::lib::environment::Environment;
use crate::lib::progress_bar::ProgressBar;
use crate::util;
use dfx_core;
use dfx_core::config::cache::{
    binary_command_from_version, delete_version, get_bin_cache, get_binary_path_from_version,
    is_version_installed,
};
use dfx_core::error::cache::{
    DeleteCacheError, GetBinaryCommandPathError, InstallCacheError, IsCacheInstalledError,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use semver::Version;
use slog::info;
use std::io::{stderr, IsTerminal};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

// POSIX permissions for files in the cache.
#[cfg(unix)]
const EXEC_READ_USER_ONLY_PERMISSION: u32 = 0o500;
#[cfg(unix)]
const READ_USER_ONLY_PERMISSION: u32 = 0o400;

#[derive(Debug, Clone)]
pub struct VersionCache {
    version: Version,
}

impl VersionCache {
    pub fn with_version(version: &Version) -> VersionCache {
        VersionCache {
            version: version.clone(),
        }
    }
    pub fn install(env: &dyn Environment, version: &str) -> Result<(), InstallCacheError> {
        install_version(env, version, false).map(|_| {})
    }
    pub fn force_install(env: &dyn Environment, version: &str) -> Result<(), InstallCacheError> {
        install_version(env, version, true).map(|_| {})
    }

    pub fn version_str(&self) -> String {
        format!("{}", self.version)
    }

    #[allow(dead_code)]
    pub fn is_installed(&self) -> Result<bool, IsCacheInstalledError> {
        is_version_installed(&self.version_str())
    }

    pub fn delete(&self) -> Result<(), DeleteCacheError> {
        delete_version(&self.version_str()).map(|_| {})
    }

    pub fn get_binary_command_path(
        &self,
        env: &dyn Environment,
        binary_name: &str,
    ) -> Result<PathBuf, GetBinaryCommandPathError> {
        Self::install(env, &self.version_str())?;
        let path = get_binary_path_from_version(&self.version_str(), binary_name)?;
        Ok(path)
    }

    pub fn get_binary_command(
        &self,
        env: &dyn Environment,
        binary_name: &str,
    ) -> Result<std::process::Command, GetBinaryCommandPathError> {
        Self::install(env, &self.version_str())?;
        let path = binary_command_from_version(&self.version_str(), binary_name)?;
        Ok(path)
    }
}

pub fn install_version(
    env: &dyn Environment,
    v: &str,
    force: bool,
) -> Result<PathBuf, InstallCacheError> {
    let p = get_bin_cache(v)?;
    if !force && is_version_installed(v).unwrap_or(false) {
        return Ok(p);
    }

    if Version::parse(v).map_err(|e| InstallCacheError::MalformedSemverString(v.to_string(), e))?
        == *dfx_version()
    {
        // Dismiss as fast as possible. We use the current_exe variable after an
        // expensive step, and if this fails we can't continue anyway.
        let current_exe = dfx_core::foundation::get_current_exe()?;

        let b: Option<ProgressBar> = if stderr().is_terminal() {
            let b = env.new_spinner(format!("Installing version {v} of dfx...").into());
            Some(b)
        } else {
            None
        };

        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(|byte| byte as char)
            .collect();
        let temp_p = get_bin_cache(&format!("_{}_{}", v, rand_string))?;
        dfx_core::fs::create_dir_all(&temp_p)?;

        let mut binary_cache_assets =
            util::assets::binary_cache().map_err(InstallCacheError::ReadBinaryCacheStoreFailed)?;
        // Write binaries and set them to be executable.
        for file in binary_cache_assets
            .entries()
            .map_err(InstallCacheError::ReadBinaryCacheEntriesFailed)?
        {
            let mut file = file.map_err(InstallCacheError::ReadBinaryCacheEntryFailed)?;

            if file.header().entry_type().is_dir() {
                continue;
            }
            dfx_core::fs::tar_unpack_in(temp_p.as_path(), &mut file)?;
            // On *nix we need to set the execute permission as the tgz doesn't include it
            #[cfg(unix)]
            {
                let archive_path = dfx_core::fs::get_archive_path(&file)?;
                let mode = if archive_path.starts_with("base/") {
                    READ_USER_ONLY_PERMISSION
                } else {
                    EXEC_READ_USER_ONLY_PERMISSION
                };
                let full_path = temp_p.join(archive_path);
                let mut perms = dfx_core::fs::read_permissions(full_path.as_path())?;
                perms.set_mode(mode);
                dfx_core::fs::set_permissions(full_path.as_path(), perms)?;
            }
        }

        // Copy our own binary in the cache.
        let dfx = temp_p.join("dfx");
        #[allow(clippy::needless_borrows_for_generic_args)]
        dfx_core::fs::write(&dfx, dfx_core::fs::read(&current_exe)?)?;
        // On *nix we need to set the execute permission as the tgz doesn't include it
        #[cfg(unix)]
        {
            let mut perms = dfx_core::fs::read_permissions(&dfx)?;
            perms.set_mode(EXEC_READ_USER_ONLY_PERMISSION);
            dfx_core::fs::set_permissions(&dfx, perms)?;
        }

        // atomically install cache version into place
        if force && p.exists() {
            dfx_core::fs::remove_dir_all(&p)?;
        }

        if dfx_core::fs::rename(temp_p.as_path(), &p).is_ok() {
            if let Some(b) = b {
                b.finish_and_clear();
                if force {
                    info!(env.get_logger(), "Installed dfx {v} to cache.");
                }
            }
        } else {
            dfx_core::fs::remove_dir_all(temp_p.as_path())?;
            if let Some(b) = b {
                b.finish_and_clear();
                info!(env.get_logger(), "dfx {v} was already installed in cache.");
            }
        }
        Ok(p)
    } else {
        Err(InstallCacheError::InvalidCacheForDfxVersion(v.to_owned()))
    }
}
