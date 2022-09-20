//! Code for executing `dfx sns config create`
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::error::DfxResult;
use crate::lib::sns::sns_cli::call_sns_cli;
use crate::Environment;

/// Executes `dfx sns config create`
#[context("Failed to create sns config at {}.", path.display())]
pub fn create_config(env: &dyn Environment, path: &Path) -> DfxResult {
    let args = vec![
        OsString::from("init-config-file"),
        OsString::from("--init-config-file-path"),
        OsString::from(path),
        OsString::from("new"),
    ];
    call_sns_cli(env, &args)?;
    Ok(())
}
