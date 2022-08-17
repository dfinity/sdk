use crate::config::dfx_version;
use crate::lib::error::{CacheError, DfxError, DfxResult};
use crate::util;

use anyhow::{bail, Context};
use fn_error_context::context;
use indicatif::{ProgressBar, ProgressDrawTarget};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use semver::Version;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::ExitStatus;

// POSIX permissions for files in the cache.
const EXEC_READ_USER_ONLY_PERMISSION: u32 = 0o500;

pub trait Cache {
    fn version_str(&self) -> String;
    fn is_installed(&self) -> DfxResult<bool>;
    fn install(&self) -> DfxResult;
    fn force_install(&self) -> DfxResult;
    fn delete(&self) -> DfxResult;
    fn get_binary_command_path(&self, binary_name: &str) -> DfxResult<PathBuf>;
    fn get_binary_command(&self, binary_name: &str) -> DfxResult<std::process::Command>;
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

    fn is_installed(&self) -> DfxResult<bool> {
        is_version_installed(&self.version_str())
    }

    fn install(&self) -> DfxResult {
        install_version(&self.version_str(), false).map(|_| {})
    }
    fn force_install(&self) -> DfxResult {
        install_version(&self.version_str(), true).map(|_| {})
    }

    fn delete(&self) -> DfxResult {
        delete_version(&self.version_str()).map(|_| {})
    }

    fn get_binary_command_path(&self, binary_name: &str) -> DfxResult<PathBuf> {
        get_binary_path_from_version(&self.version_str(), binary_name)
    }

    fn get_binary_command(&self, binary_name: &str) -> DfxResult<std::process::Command> {
        binary_command_from_version(&self.version_str(), binary_name)
    }
}

#[context("Failed to get cache root.")]
pub fn get_cache_root() -> DfxResult<PathBuf> {
    let cache_root = std::env::var("DFX_CACHE_ROOT").ok();
    let home =
        std::env::var("HOME").map_err(|_| DfxError::new(CacheError::CannotFindHomeDirectory()))?;
    let root = cache_root.unwrap_or(home);
    let p = PathBuf::from(root).join(".cache").join("dfinity");
    if !p.exists() {
        if let Err(_e) = std::fs::create_dir_all(&p) {
            return Err(DfxError::new(CacheError::CannotCreateCacheDirectory(p)));
        }
    } else if !p.is_dir() {
        return Err(DfxError::new(CacheError::CannotFindCacheDirectory(p)));
    }
    Ok(p)
}

/// Return the binary cache root. It constructs it if not present
/// already.
#[context("Failed to get path to binary cache root.")]
pub fn get_bin_cache_root() -> DfxResult<PathBuf> {
    let p = get_cache_root()?.join("versions");

    if !p.exists() {
        if let Err(_e) = std::fs::create_dir_all(&p) {
            return Err(DfxError::new(CacheError::CannotCreateCacheDirectory(p)));
        }
    } else if !p.is_dir() {
        return Err(DfxError::new(CacheError::CannotFindCacheDirectory(p)));
    }

    Ok(p)
}

#[context("Failed to get path to binary cache for version '{}'.", v)]
pub fn get_bin_cache(v: &str) -> DfxResult<PathBuf> {
    let root = get_bin_cache_root()?;
    Ok(root.join(v))
}

#[context("Failed to determine if cache is installed for version '{}'.", v)]
pub fn is_version_installed(v: &str) -> DfxResult<bool> {
    get_bin_cache(v).map(|c| c.is_dir())
}

pub fn delete_version(v: &str) -> DfxResult<bool> {
    if !is_version_installed(v).unwrap_or(false) {
        return Ok(false);
    }

    let root = get_bin_cache(v)?;
    std::fs::remove_dir_all(&root).with_context(|| {
        format!(
            "Failed to remove bin cache root {}.",
            root.to_string_lossy()
        )
    })?;

    Ok(true)
}

#[context("Failed to install binary cache for version '{}'.", v)]
pub fn install_version(v: &str, force: bool) -> DfxResult<PathBuf> {
    let p = get_bin_cache(v)?;
    if !force && is_version_installed(v).unwrap_or(false) {
        return Ok(p);
    }

    if Version::parse(v).with_context(|| format!("Failed to parse version string {}.", v))?
        == *dfx_version()
    {
        // Dismiss as fast as possible. We use the current_exe variable after an
        // expensive step, and if this fails we can't continue anyway.
        let current_exe =
            std::env::current_exe().context("Failed to identify currently running executable.")?;

        let b: Option<ProgressBar> = if atty::is(atty::Stream::Stderr) {
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
        let temp_p = get_bin_cache(&format!("_{}_{}", v, rand_string))
            .context("Failed to get temporary bin cache path.")?;
        std::fs::create_dir(&temp_p).with_context(|| {
            format!(
                "Failed to create temporary bin cache dir {}.",
                temp_p.to_string_lossy()
            )
        })?;

        let mut binary_cache_assets =
            util::assets::binary_cache().context("Failed to get asset binary cache.")?;
        // Write binaries and set them to be executable.
        for file in binary_cache_assets
            .entries()
            .context("Failed to get binary cache archive entires.")?
        {
            let mut file = file.context("Failed to get binary cache archive entry.")?;

            if file.header().entry_type().is_dir() {
                continue;
            }
            file.unpack_in(temp_p.as_path())
                .context("Failed to unpack archive asset.")?;

            let full_path = temp_p.join(file.path().context("Failed to get file path.")?);
            let mut perms = std::fs::metadata(full_path.as_path())
                .with_context(|| {
                    format!(
                        "Failed to get file metadata for {}.",
                        full_path.to_string_lossy()
                    )
                })?
                .permissions();
            perms.set_mode(EXEC_READ_USER_ONLY_PERMISSION);
            std::fs::set_permissions(full_path.as_path(), perms).with_context(|| {
                format!(
                    "Failed to set file permissions for {}.",
                    full_path.to_string_lossy()
                )
            })?;
        }

        // Copy our own binary in the cache.
        let dfx = temp_p.join("dfx");
        std::fs::write(
            &dfx,
            std::fs::read(current_exe).context("Failed to read currently running executable.")?,
        )
        .with_context(|| {
            format!(
                "Failed to copy running binary {} to cache.",
                dfx.to_string_lossy()
            )
        })?;
        // And make it executable.
        let mut perms = std::fs::metadata(&dfx)
            .with_context(|| {
                format!(
                    "Failed to read file metadata for {}.",
                    dfx.to_string_lossy()
                )
            })?
            .permissions();
        perms.set_mode(EXEC_READ_USER_ONLY_PERMISSION);
        std::fs::set_permissions(&dfx, perms).with_context(|| {
            format!("Failed to set file metadata for {}.", dfx.to_string_lossy())
        })?;

        // atomically install cache version into place
        if force && p.exists() {
            std::fs::remove_dir_all(&p)
                .with_context(|| format!("Failed to remove {}.", p.to_string_lossy()))?;
        }

        if std::fs::rename(&temp_p, &p).is_ok() {
            if let Some(b) = b {
                b.finish_with_message(format!("Version v{} installed successfully.", v));
            }
        } else {
            std::fs::remove_dir_all(&temp_p).with_context(|| {
                format!(
                    "Failed to remove temp binary cache {}.",
                    temp_p.to_string_lossy()
                )
            })?;
            if let Some(b) = b {
                b.finish_with_message(format!("Version v{} was already installed.", v));
            }
        }

        Ok(p)
    } else {
        Err(DfxError::new(CacheError::UnknownVersion(v.to_owned())))
    }
}

#[context(
    "Failed to get path to binary '{}' for version '{}'.",
    binary_name,
    version
)]
pub fn get_binary_path_from_version(version: &str, binary_name: &str) -> DfxResult<PathBuf> {
    install_version(version, false)?;

    let env_var_name = format!("DFX_{}_PATH", binary_name.replace('-', "_").to_uppercase());

    if let Ok(path) = std::env::var(env_var_name) {
        return Ok(PathBuf::from(path));
    }

    Ok(get_bin_cache(version)?.join(binary_name))
}

#[context("Failed to get binary '{}' for version '{}'.", name, version)]
pub fn binary_command_from_version(version: &str, name: &str) -> DfxResult<std::process::Command> {
    let path = get_binary_path_from_version(version, name)?;
    let cmd = std::process::Command::new(path);

    Ok(cmd)
}

#[context("Failed to list cache versions.")]
pub fn list_versions() -> DfxResult<Vec<Version>> {
    let root = get_bin_cache_root()?;
    let mut result: Vec<Version> = Vec::new();

    for entry in std::fs::read_dir(&root).with_context(|| {
        format!(
            "Failed to read bin cache root content at {}.",
            root.to_string_lossy()
        )
    })? {
        let entry = entry.with_context(|| {
            format!(
                "Failed to read an entry in bin cache root at {}.",
                root.to_string_lossy()
            )
        })?;
        if let Some(version) = entry.file_name().to_str() {
            if version.starts_with('_') {
                // temp directory for version being installed
                continue;
            }
            result.push(
                Version::parse(version)
                    .with_context(|| format!("Failed to parse version from {}.", version))?,
            );
        }
    }

    Ok(result)
}

pub fn call_cached_dfx(v: &Version) -> DfxResult<ExitStatus> {
    let v = format!("{}", v);
    let command_path = get_binary_path_from_version(&v, "dfx")?;
    if command_path
        == std::env::current_exe().context("Failed to get currently running executable.")?
    {
        bail!("Invalid cache for version {}.", v)
    }

    std::process::Command::new(command_path)
        .args(std::env::args().skip(1))
        .status()
        .map_err(DfxError::from)
}
