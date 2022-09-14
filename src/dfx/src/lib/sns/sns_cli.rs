use anyhow::{anyhow, Context};
use fn_error_context::context;
use std::ffi::OsStr;
use std::process::{self, Command};

use crate::lib::error::DfxResult;
use crate::Environment;

/// Calls the sns cli tool from the SNS codebase in the ic repo.
#[context("Failed to call sns CLI.")]
pub fn call_sns_cli<S, I>(env: &dyn Environment, args: I) -> DfxResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let cli_name = "sns";
    let sns_cli = env
        .get_cache()
        .get_binary_command_path(cli_name)
        .with_context(|| format!("Could not find bundled binary '{cli_name}'."))?;
    let mut command = Command::new(sns_cli);
    command.args(args);
    command
        .stdin(process::Stdio::null())
        .output()
        .map_err(anyhow::Error::from)
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(anyhow!(
                    "SNS cli call failed:\n{:?} {:?}\nStdout:\n{:?}\n\nStderr:\n{:?}",
                    command.get_program(),
                    command.get_args(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        })?;
    Ok(())
}
