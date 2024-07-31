use crate::config::dfx_version;
use crate::util;
use dfx_core;
use dfx_core::config::cache::{
    binary_command_from_version, delete_version, get_bin_cache, get_binary_path_from_version,
    is_version_installed, Cache,
};
use dfx_core::error::cache::CacheError;
use indicatif::{ProgressBar, ProgressDrawTarget};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use semver::Version;
use std::io::{stderr, IsTerminal};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

// POSIX permissions for files in the cache.
#[cfg(unix)]
const EXEC_READ_USER_ONLY_PERMISSION: u32 = 0o500;
#[cfg(unix)]
const READ_USER_ONLY_PERMISSION: u32 = 0o400;

pub struct DiskBasedCache {
    version: Version,
}

impl DiskBasedCache {
    pub fn with_version(version: &Version) -> DiskBasedCache {
        DiskBasedCache {
            version: version.clone(),
        }
    }
    pub fn install(version: &str) -> Result<(), CacheError> {
        install_version(version, false).map(|_| {})
    }
    pub fn force_install(version: &str) -> Result<(), CacheError> {
        install_version(version, true).map(|_| {})
    }
}

#[allow(dead_code)]
impl Cache for DiskBasedCache {
    fn version_str(&self) -> String {
        format!("{}", self.version)
    }

    fn is_installed(&self) -> Result<bool, CacheError> {
        is_version_installed(&self.version_str())
    }

    fn delete(&self) -> Result<(), CacheError> {
        delete_version(&self.version_str()).map(|_| {})
    }

    fn get_binary_command_path(&self, binary_name: &str) -> Result<PathBuf, CacheError> {
        Self::install(&self.version_str())?;
        get_binary_path_from_version(&self.version_str(), binary_name)
    }

    fn get_binary_command(&self, binary_name: &str) -> Result<std::process::Command, CacheError> {
        Self::install(&self.version_str())?;
        binary_command_from_version(&self.version_str(), binary_name)
    }
}

pub fn install_version(v: &str, force: bool) -> Result<PathBuf, CacheError> {
    let p = get_bin_cache(v)?;
    if !force && is_version_installed(v).unwrap_or(false) {
        return Ok(p);
    }

    if Version::parse(v).map_err(|e| CacheError::MalformedSemverString(v.to_string(), e))?
        == *dfx_version()
    {
        // Dismiss as fast as possible. We use the current_exe variable after an
        // expensive step, and if this fails we can't continue anyway.
        let current_exe = dfx_core::foundation::get_current_exe()?;

        let b: Option<ProgressBar> = if stderr().is_terminal() {
            let b = ProgressBar::new_spinner();
            b.set_draw_target(ProgressDrawTarget::stderr());
            b.set_message(format!("Installing version {} of dfx...", v));
            b.enable_steady_tick(80);
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
            util::assets::binary_cache().map_err(CacheError::ReadBinaryCacheStoreFailed)?;
        // Write binaries and set them to be executable.
        for file in binary_cache_assets
            .entries()
            .map_err(CacheError::ReadBinaryCacheEntriesFailed)?
        {
            let mut file = file.map_err(CacheError::ReadBinaryCacheEntryFailed)?;

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
                b.finish_with_message(format!("Installed dfx {} to cache.", v));
            }
        } else {
            dfx_core::fs::remove_dir_all(temp_p.as_path())?;
            if let Some(b) = b {
                b.finish_with_message(format!("dfx {} was already installed in cache.", v));
            }
        }
        Ok(p)
    } else {
        Err(CacheError::InvalidCacheForDfxVersion(v.to_owned()))
    }
}
