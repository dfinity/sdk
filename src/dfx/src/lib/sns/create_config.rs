//! Code for creating SNS configurations
use anyhow::{anyhow, Context};
use fn_error_context::context;
use std::path::Path;
use std::process::{self, Command};

use crate::lib::error::DfxResult;
use crate::Environment;

/// Ceates an SNS configuration template.
#[context("Failed to create sns config at {}.", path.display())]
pub fn create_config(env: &dyn Environment, path: &Path) -> DfxResult {
    let cli_name = "sns";
    let sns_cli = env
        .get_cache()
        .get_binary_command_path(cli_name)
        .with_context(|| format!("Could not find bundled binary '{cli_name}'."))?;
    let mut command = Command::new(sns_cli);
    command
        .arg("init-config-file")
        .arg("--init-config-file-path")
        .arg(path)
        .arg("new");
    command
        .stdin(process::Stdio::null())
        .output()
        .map_err(anyhow::Error::from)
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(anyhow!(
                    "Failed to create an SNS configuration.\n{:?} {:?}\nStdout:\n{:?}\n\nStderr:\n{:?}",
                    command.get_program(),
                    command.get_args(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        })?;
    Ok(())
}
