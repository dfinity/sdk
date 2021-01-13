use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::error::DfxResult;

use anyhow::bail;
use atty::Stream;
use std::fs::File;
use std::include_str;
use std::path::PathBuf;

pub fn get_telemetry_config_root() -> DfxResult<PathBuf> {
    let root = get_config_dfx_dir_path()?.join("telemetry");
    if !root.exists() {
        if std::fs::create_dir_all(&root).is_err() {
            bail!(
                "Cannot create telemetry config directory at '{}'.",
                root.display(),
            );
        }
    } else if !root.is_dir() {
        bail!(
            "Cannot find telemetry config  directory at '{}'.",
            root.display(),
        );
    }
    Ok(root)
}

pub fn witness_telemetry_consent() -> DfxResult<()> {
    if atty::is(Stream::Stderr) {
        let file = get_telemetry_config_root()?.join("witness.blank");
        if !file.exists() {
            eprintln!("\n{}", include_str!("consent.txt"));
            if File::create(&file).is_err() {
                bail!(
                    "Cannot create telemetry consent witness file at '{}'.",
                    file.display(),
                );
            }
        } else if !file.is_file() {
            bail!(
                "Cannot find telemetry consent witness file at '{}'.",
                file.display(),
            );
        }
    }
    Ok(())
}
