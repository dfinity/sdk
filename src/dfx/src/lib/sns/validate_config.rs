//! Code for checking SNS config file validity
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::call_bundled::wsl_call_bundled;
use crate::lib::error::DfxResult;
use crate::util::wsl_path;
use crate::Environment;

/// Checks whether an SNS configuration file is valid.
#[context("Failed to validate SNS config at {}.", path.display())]
pub fn validate_config(env: &dyn Environment, path: &Path) -> DfxResult<String> {
    let args = vec![
        OsString::from("init-config-file"),
        OsString::from("--init-config-file-path"),
        OsString::from(wsl_path(path)?),
        OsString::from("validate"),
    ];
    wsl_call_bundled(env, "sns", &args)
        .map(|_| format!("SNS config file is valid: {}", path.display()))
}
