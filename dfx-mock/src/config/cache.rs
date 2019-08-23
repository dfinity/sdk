use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;

use crate::util;
use crate::config::dfinity::Config;

pub fn get_bin_cache_root() -> Result<PathBuf> {
    let home = match std::env::var("HOME") {
        Ok(h) => Ok(h),
        Err(_) => Err(Error::new(ErrorKind::Other, "Could not find the HOME directory.")),
    }?;

    let p = PathBuf::from(home)
        .join(".cache")
        .join("dfinity")
        .join("versions")
        ;

    if !p.exists() {
        std::fs::create_dir_all(&p)?;
    } else if !p.is_dir() {
        return Err(Error::new(ErrorKind::Other, "Cache root is not a directory."));
    }

    Ok(p)
}

pub fn get_bin_cache(v: &str) -> Result<PathBuf> {
    let root = get_bin_cache_root()?;

    match v {
        "v0.1.0" | "0.1.0" => Ok(root.join("v0.1.0")),
        v => Err(Error::new(ErrorKind::Other, format!("Unknown version: {}", v))),
    }
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

    match v {
        "v0.1.0" | "0.1.0" => {
            util::assets_v0_1_0_files()?.unpack(p.as_path())?;
            Ok(p)
        },
        v => Err(Error::new(ErrorKind::Other, format!("Unknown version: {}", v))),
    }
}


pub fn get_binary_path_from_config(config: &Config, binary_name: &str) -> Result<PathBuf> {
    let version = config.get_config().get_dfx_version();

    Ok(get_bin_cache(version.as_str())?.join(binary_name))
}

pub fn binary_command(config: &Config, name: &str) -> Result<std::process::Command> {
    let path = get_binary_path_from_config(config, name);

    match name {
        "asc" => Ok(
            std::process::Command::new(path)
                .env("ASC_RTS", get_binary_path_from_config(config, "as-rts.wasm"))
        )
    }
    let mut cmd = std::process::Command::new(path);

    Ok(cmd)
}
