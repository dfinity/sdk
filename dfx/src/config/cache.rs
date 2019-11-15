use crate::config::dfx_version;
use crate::lib::error::DfxError::CacheError;
use crate::lib::error::{CacheErrorKind, DfxResult};
use crate::util;
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn get_bin_cache_root() -> DfxResult<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| CacheError(CacheErrorKind::CannotFindUserHomeDirectory()))?;

    let p = PathBuf::from(home)
        .join(".cache")
        .join("dfinity")
        .join("versions");

    if !p.exists() {
        if let Err(e) = std::fs::create_dir_all(&p) {
            return Err(CacheError(CacheErrorKind::CannotCreateCacheDirectory(p, e)));
        }
    } else if !p.is_dir() {
        return Err(CacheError(CacheErrorKind::CacheShouldBeADirectory(p)));
    }

    Ok(p)
}

pub fn get_bin_cache(v: &str) -> DfxResult<PathBuf> {
    let root = get_bin_cache_root()?;
    Ok(root.join(v))
}

pub fn is_version_installed(v: &str) -> DfxResult<bool> {
    get_bin_cache(v).and_then(|c| Ok(c.is_dir()))
}

pub fn install_version(v: &str) -> DfxResult<PathBuf> {
    let p = get_bin_cache(v)?;
    if is_version_installed(v).unwrap_or(false) {
        return Ok(p);
    }

    if v == dfx_version() {
        let b: Option<ProgressBar> = if atty::is(atty::Stream::Stderr) {
            let b = ProgressBar::new_spinner();
            b.set_draw_target(ProgressDrawTarget::stderr());
            b.set_message(&format!("Installing version {} of dfx...", v));
            b.enable_steady_tick(80);
            Some(b)
        } else {
            None
        };

        let mut binary_cache_assets = util::assets::binary_cache()?;
        // Write binaries and set them to be executable.
        for file in binary_cache_assets.entries()? {
            let mut file = file?;

            if file.header().entry_type().is_dir() {
                continue;
            }
            file.unpack_in(p.as_path())?;

            let full_path = p.join(file.path()?);
            let mut perms = std::fs::metadata(full_path.as_path())?.permissions();
            perms.set_mode(0o554);
            std::fs::set_permissions(full_path.as_path(), perms)?;
        }

        if let Some(b) = b {
            b.finish_with_message(&format!("Version v{} installed successfully.", v));
        }

        Ok(p)
    } else {
        Err(CacheError(CacheErrorKind::UnknownDfxVersion(v.to_owned())))
    }
}

pub fn get_binary_path_from_version(version: &str, binary_name: &str) -> DfxResult<PathBuf> {
    install_version(version)?;

    Ok(get_bin_cache(version)?.join(binary_name))
}

pub fn binary_command_from_version(version: &str, name: &str) -> DfxResult<std::process::Command> {
    let path = get_binary_path_from_version(version, name)?;
    let cmd = std::process::Command::new(path);

    Ok(cmd)
}
