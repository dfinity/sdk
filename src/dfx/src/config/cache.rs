use crate::config::dfx_version;
use crate::lib::error::CacheError;
use crate::util;
#[cfg(windows)]
use dfx_core::config::directories::project_dirs;

use dfx_core::foundation::get_user_home;
use indicatif::{ProgressBar, ProgressDrawTarget};
use semver::Version;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::ExitStatus;
use tempfile;

// POSIX permissions for files in the cache.
#[cfg(unix)]
const EXEC_READ_USER_ONLY_PERMISSION: u32 = 0o500;

pub trait Cache {
    fn version_str(&self) -> String;
    fn is_installed(&self) -> Result<bool, CacheError>;
    fn install(&self) -> Result<(), CacheError>;
    fn force_install(&self) -> Result<(), CacheError>;
    fn delete(&self) -> Result<(), CacheError>;
    fn get_binary_command_path(&self, binary_name: &str) -> Result<PathBuf, CacheError>;
    fn get_binary_command(&self, binary_name: &str) -> Result<std::process::Command, CacheError>;
}

pub struct DiskBasedCache {
    version: Version,
}

impl DiskBasedCache {
    pub fn with_version(version: &Version) -> DiskBasedCache {
        DiskBasedCache {
            version: version.clone(),
        }
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

    fn install(&self) -> Result<(), CacheError> {
        install_version(&self.version_str(), false).map(|_| {})
    }
    fn force_install(&self) -> Result<(), CacheError> {
        install_version(&self.version_str(), true).map(|_| {})
    }
    fn delete(&self) -> Result<(), CacheError> {
        delete_version(&self.version_str()).map(|_| {})
    }

    fn get_binary_command_path(&self, binary_name: &str) -> Result<PathBuf, CacheError> {
        get_binary_path_from_version(&self.version_str(), binary_name)
    }

    fn get_binary_command(&self, binary_name: &str) -> Result<std::process::Command, CacheError> {
        binary_command_from_version(&self.version_str(), binary_name)
    }
}

pub fn get_cache_root() -> Result<PathBuf, CacheError> {
    let cache_root = std::env::var_os("DFX_CACHE_ROOT");
    // dirs-next is not used for *nix to preserve existing paths
    #[cfg(not(windows))]
    let p = {
        let home = get_user_home()?;
        let root = cache_root.unwrap_or(home);
        PathBuf::from(root).join(".cache").join("dfinity")
    };
    #[cfg(windows)]
    let p = match cache_root {
        Some(var) => PathBuf::from(var),
        None => project_dirs()?.cache_dir().to_owned(),
    };
    if !p.exists() {
        if let Err(_e) = dfx_core::fs::create_dir_all(&p) {
            return Err(CacheError::CreateCacheDirectoryFailed(p));
        }
    } else if !p.is_dir() {
        return Err(CacheError::FindCacheDirectoryFailed(p));
    }
    Ok(p)
}

/// Return the binary cache root. It constructs it if not present
/// already.
pub fn get_bin_cache_root() -> Result<PathBuf, CacheError> {
    let p = get_cache_root()?.join("versions");

    if !p.exists() {
        if let Err(_e) = dfx_core::fs::create_dir_all(&p) {
            return Err(CacheError::CreateCacheDirectoryFailed(p));
        }
    } else if !p.is_dir() {
        return Err(CacheError::FindCacheDirectoryFailed(p));
    }

    Ok(p)
}

pub fn get_bin_cache(v: &str) -> Result<PathBuf, CacheError> {
    let root = get_bin_cache_root()?;
    Ok(root.join(v))
}

pub fn is_version_installed(v: &str) -> Result<bool, CacheError> {
    get_bin_cache(v).map(|c| c.is_dir())
}

pub fn delete_version(v: &str) -> Result<bool, CacheError> {
    if !is_version_installed(v).unwrap_or(false) {
        return Ok(false);
    }

    let root = get_bin_cache(v)?;
    dfx_core::fs::remove_dir_all(&root)?;

    Ok(true)
}

pub fn install_version(v: &str, force: bool) -> Result<PathBuf, CacheError> {
    let p = get_bin_cache(v)?;
    if !force && is_version_installed(v).unwrap_or(false) {
        return Ok(p);
    }

    let parsed_version = Version::parse(v).map_err(CacheError::MalformedSemverVersion)?;
    if parsed_version != *dfx_version() {
        return Err(CacheError::UnknownVersion(v.to_owned()));
    };
    // Dismiss as fast as possible. We use the current_exe variable after an
    // expensive step, and if this fails we can't continue anyway.
    let current_exe = std::env::current_exe()?;

    let b: Option<ProgressBar> = if atty::is(atty::Stream::Stderr) {
        let b = ProgressBar::new_spinner();
        b.set_draw_target(ProgressDrawTarget::stderr());
        b.set_message(format!("Installing version {} of dfx...", v));
        b.enable_steady_tick(80);
        Some(b)
    } else {
        None
    };

    let bin_cache_path = get_bin_cache_root()?;
    let tempdir = tempfile::tempdir_in(bin_cache_path)?;
    let temp_p = tempdir.path();

    let mut binary_cache_assets = util::assets::binary_cache()?;
    // Write binaries and set them to be executable.
    for file in binary_cache_assets.entries()? {
        let mut file = file?;

        if file.header().entry_type().is_dir() {
            continue;
        }
        file.unpack_in(temp_p)?;
        // On *nix we need to set the execute permission as the tgz doesn't include it
        #[cfg(unix)]
        {
            let full_path = temp_p.join(file.path()?);
            let mut perms = dfx_core::fs::read_permissions(full_path.as_path())?;
            perms.set_mode(EXEC_READ_USER_ONLY_PERMISSION);
            dfx_core::fs::set_permissions(full_path.as_path(), perms)?;
        }
    }

    // Copy our own binary in the cache.
    let dfx = temp_p.join("dfx");
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

    if dfx_core::fs::rename(&temp_p, &p).is_ok() {
        if let Some(b) = b {
            b.finish_with_message(format!("Version v{} installed successfully.", v));
        }
    } else {
        dfx_core::fs::remove_dir_all(&temp_p)?;
        if let Some(b) = b {
            b.finish_with_message(format!("Version v{} was already installed.", v));
        }
    }

    Ok(p)
}

pub fn get_binary_path_from_version(
    version: &str,
    binary_name: &str,
) -> Result<PathBuf, CacheError> {
    install_version(version, false)?;

    let env_var_name = format!("DFX_{}_PATH", binary_name.replace('-', "_").to_uppercase());

    if let Ok(path) = std::env::var(env_var_name) {
        return Ok(PathBuf::from(path));
    }

    Ok(get_bin_cache(version)?.join(binary_name))
}

pub fn binary_command_from_version(
    version: &str,
    name: &str,
) -> Result<std::process::Command, CacheError> {
    let path = get_binary_path_from_version(version, name)?;
    let cmd = std::process::Command::new(path);

    Ok(cmd)
}

pub fn list_versions() -> Result<Vec<Version>, CacheError> {
    let root = get_bin_cache_root()?;
    let mut result: Vec<Version> = Vec::new();

    for entry in dfx_core::fs::read_dir(&root)? {
        let entry = entry?;
        if let Some(version) = entry.file_name().to_str() {
            if version.starts_with('_') {
                // temp directory for version being installed
                continue;
            }
            result.push(Version::parse(version).map_err(CacheError::MalformedSemverVersion)?);
        }
    }

    Ok(result)
}

pub fn call_cached_dfx(v: &Version) -> Result<ExitStatus, CacheError> {
    let v = format!("{}", v);
    let command_path = get_binary_path_from_version(&v, "dfx")?;
    if command_path
        == std::env::current_exe()?
    {
        return Err(CacheError::UnknownVersion(v));
    }

    std::process::Command::new(command_path)
        .args(std::env::args().skip(1))
        .status().map_err(CacheError::StdIoError)
}
