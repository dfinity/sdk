use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::error::DfxResult;
use crate::lib::sns::sns_cli::call_sns_cli;
use crate::Environment;

///
#[context("Failed to validate SNS config at {}.", path.display())]
pub fn validate_config(env: &dyn Environment, path: &Path) -> DfxResult {
    let args = vec![OsString::from("init-config-file"), OsString::from("--init-config-file-path"), OsString::from(path), OsString::from("validate")];
    call_sns_cli(env, &args)
}
