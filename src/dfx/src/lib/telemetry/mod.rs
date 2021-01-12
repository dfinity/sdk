use crate::lib::config::get_config_dfx_dir_path;
use crate::lib::error::DfxResult;

use anyhow::bail;
use libc::{isatty, STDOUT_FILENO};
use std::fs::File;
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
    let file = get_telemetry_config_root()?.join("witness.blank");
    if !file.exists() {
        if File::create(&file).is_err() {
            bail!(
                "Cannot create telemetry consent witness file at '{}'.",
                file.display(),
            );
        }
        let is_tty = unsafe { isatty(STDOUT_FILENO as i32) } != 0;
        if is_tty {
            eprintln!("\nThe DFINITY Canister SDK sends anonymous usage data to DFINITY Stiftung by\ndefault. If you wish to disable this behavior, then please set the environment\nvariable DFX_TELEMETRY_DISABLED=1. Learn more at https://sdk.dfinity.org.\n");
        }
    } else if !file.is_file() {
        bail!(
            "Cannot find telemetry consent witness file at '{}'.",
            file.display(),
        );
    }
    Ok(())
}
