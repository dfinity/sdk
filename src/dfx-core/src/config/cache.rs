#[cfg(windows)]
use crate::config::directories::project_dirs;
use crate::error::cache::CacheError;
#[cfg(not(windows))]
use crate::foundation::get_user_home;
use semver::Version;
use std::{path::PathBuf, process::ExitStatus};

pub trait Cache {
    fn version_str(&self) -> String;
    fn is_installed(&self) -> Result<bool, CacheError>;
    fn delete(&self) -> Result<(), CacheError>;
    fn get_binary_command_path(&self, binary_name: &str) -> Result<PathBuf, CacheError>;
    fn get_binary_command(&self, binary_name: &str) -> Result<std::process::Command, CacheError>;
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
    if p.exists() && !p.is_dir() {
        return Err(CacheError::FindCacheDirectoryFailed(p));
    }
    Ok(p)
}

/// Constructs and returns <cache root>/versions/<version> without creating any directories.
pub fn get_cache_path_for_version(v: &str) -> Result<PathBuf, CacheError> {
    let p = get_cache_root()?.join("versions").join(v);
    Ok(p)
}

/// Return the binary cache root. It constructs it if not present
/// already.
pub fn get_bin_cache_root() -> Result<PathBuf, CacheError> {
    let p = get_cache_root()?.join("versions");

    if !p.exists() {
        crate::fs::create_dir_all(&p).map_err(CacheError::CreateCacheDirectoryFailed)?;
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
    crate::fs::remove_dir_all(&root)?;

    Ok(true)
}

pub fn get_binary_path_from_version(
    version: &str,
    binary_name: &str,
) -> Result<PathBuf, CacheError> {
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

    for entry in crate::fs::read_dir(&root)? {
        let entry = entry.map_err(CacheError::ReadCacheEntryFailed)?;
        if let Some(version) = entry.file_name().to_str() {
            if version.starts_with('_') {
                // temp directory for version being installed
                continue;
            }
            result.push(
                Version::parse(version)
                    .map_err(|e| CacheError::MalformedSemverString(version.to_string(), e))?,
            );
        }
    }

    Ok(result)
}

pub fn call_cached_dfx(v: &Version) -> Result<ExitStatus, CacheError> {
    let v = format!("{}", v);
    let command_path = get_binary_path_from_version(&v, "dfx")?;
    if command_path == crate::foundation::get_current_exe()? {
        return Err(CacheError::InvalidCacheForDfxVersion(v));
    }

    let mut binding = std::process::Command::new(command_path);
    let cmd = binding.args(std::env::args().skip(1));
    let result = crate::process::execute_process(cmd)?;
    Ok(result)
}
