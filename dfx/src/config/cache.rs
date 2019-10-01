use crate::config::dfx_version;
use crate::util;
use std::io::{Error, ErrorKind, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub fn get_bin_cache_root() -> Result<PathBuf> {
    let home = match std::env::var("HOME") {
        Ok(h) => Ok(h),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "Could not find the HOME directory.",
        )),
    }?;

    let p = PathBuf::from(home)
        .join(".cache")
        .join("dfinity")
        .join("versions");

    if !p.exists() {
        std::fs::create_dir_all(&p)?;
    } else if !p.is_dir() {
        return Err(Error::new(
            ErrorKind::Other,
            "Cache root is not a directory.",
        ));
    }

    Ok(p)
}

pub fn get_bin_cache(v: &str) -> Result<PathBuf> {
    let root = get_bin_cache_root()?;
    Ok(root.join(v))
}

pub fn is_version_installed(v: &str) -> Result<bool> {
    match get_bin_cache(v) {
        Ok(v) => Ok(v.is_dir()),
        Err(err) => {
            if err.kind() == ErrorKind::Other {
                Ok(false)
            } else {
                Err(err)
            }
        }
    }
}

pub fn install_version(v: &str) -> Result<PathBuf> {
    let p = get_bin_cache(v)?;
    if is_version_installed(v).unwrap_or(false) {
        return Ok(p);
    }

    if v == dfx_version() {
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
        Ok(p)
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!("Unknown version: {}", v),
        ))
    }
}

pub fn get_binary_path_from_version(version: &str, binary_name: &str) -> Result<PathBuf> {
    Ok(get_bin_cache(version)?.join(binary_name))
}

pub fn binary_command_from_version(version: &str, name: &str) -> Result<std::process::Command> {
    let path = get_binary_path_from_version(version, name)?;
    let cmd = std::process::Command::new(path);
    Ok(cmd)
}
