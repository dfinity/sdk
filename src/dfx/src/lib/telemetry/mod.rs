use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};

use std::fs::File;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("Cannot create telemetry directory at '{0}'.")]
    CannotCreateTelemetryDirectory(PathBuf),

    #[error("Cannot find telemetry directory at '{0}'.")]
    CannotFindTelemetryDirectory(PathBuf),

    #[error("Cannot create telemetry witness file at '{0}'.")]
    CannotCreateTelemetryWitnessFile(PathBuf),

    #[error("Cannot find telemetry witness file at '{0}'.")]
    CannotFindTelemetryWitnessFile(PathBuf),
}

pub fn get_telemetry_root(env: &dyn Environment) -> DfxResult<PathBuf> {
    let root = env.get_cache().get_binary_command_path("telemetry")?;
    if !root.exists() {
        if std::fs::create_dir_all(&root).is_err() {
            return Err(DfxError::new(
                TelemetryError::CannotCreateTelemetryDirectory(root),
            ));
        }
    } else if !root.is_dir() {
        return Err(DfxError::new(TelemetryError::CannotFindTelemetryDirectory(
            root,
        )));
    }
    Ok(root)
}

pub fn witness_telemetry_consent(env: &dyn Environment) -> DfxResult<()> {
    let file = get_telemetry_root(env)?.join("witness.blank");
    if !file.exists() {
        if File::create(&file).is_err() {
            return Err(DfxError::new(
                TelemetryError::CannotCreateTelemetryWitnessFile(file),
            ));
        }
        println!("\nThe DFINITY Canister SDK sends anonymous usage data to DFINITY Stiftung by\ndefault. If you wish to disable this behavior, then please set the environment\nvariable DFX_TELEMETRY_DISABLED=1. Learn more at https://sdk.dfinity.org.\n");
    } else if !file.is_file() {
        return Err(DfxError::new(
            TelemetryError::CannotFindTelemetryWitnessFile(file),
        ));
    }
    Ok(())
}
