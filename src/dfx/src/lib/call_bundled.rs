//! Library for calling bundled command line tools.
use anyhow::{anyhow, Context};
use fn_error_context::context;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{self, Command};

use crate::lib::error::DfxResult;
use crate::Environment;

/// Calls a bundled command line tool.
///
/// # Returns
/// - On success, returns stdout as a string.
/// - On error, returns an error message including stdout and stderr.
#[context("Failed to call sns CLI.")]
pub fn call_bundled<S, I>(env: &dyn Environment, command: &str, args: I) -> DfxResult<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = env
        .get_cache()
        .get_binary_command_path(command)
        .with_context(|| format!("Could not find bundled binary '{command}'."))?;
    let mut command = Command::new(&binary);
    command.args(args);
    // The sns command line tool itself calls dfx; it should call this dfx.
    // The sns command line tool should not rely on commands not packaged with dfx.
    // The same applies to other bundled binaries.
    command.env("PATH", binary.parent().unwrap_or_else(|| Path::new(".")));
    command
        .stdin(process::Stdio::null())
        .output()
        .map_err(anyhow::Error::from)
        .and_then(|output| {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).into_owned())
            } else {
                let args: Vec<_> = command
                    .get_args()
                    .into_iter()
                    .map(OsStr::to_string_lossy)
                    .collect();
                Err(anyhow!(
                    "Call failed:\n{:?} {}\nStdout:\n{}\n\nStderr:\n{}",
                    command.get_program(),
                    args.join(" "),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        })
}
