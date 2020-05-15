use crate::config::dfinity::Config;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use std::process::Command;

/// Package arguments for moc or mo-ide as returned by
/// a package tool like https://github.com/kritzcreek/vessel
/// or, if there is no package tool, the base library.
pub type PackageArguments = Vec<String>;

pub fn load(
    env: &dyn Environment,
    config: &Config,
    quiet: bool, // LSP needs nothing to be written to stdout
) -> DfxResult<PackageArguments> {
    let packtool = config
        .get_config()
        .get_defaults()
        .get_build()
        .get_packtool();
    if packtool.is_none() {
        let stdlib_path = env
            .get_cache()
            .get_binary_command_path("base")?
            .into_os_string()
            .into_string()
            .map_err(DfxError::CouldNotConvertOsString)?;

        let base = vec![String::from("--package"), String::from("base"), stdlib_path];
        return Ok(base);
    }

    let logger = env.get_logger();
    if !quiet {
        slog::info!(logger, "Calling package tool...");
    }

    let commandline: Vec<String> = packtool
        .unwrap()
        .split_ascii_whitespace()
        .map(String::from)
        .collect();

    let mut cmd = Command::new(commandline[0].clone());
    for arg in commandline.iter().skip(1) {
        cmd.arg(arg);
    }

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => {
            return Err(DfxError::FailedToInvokePackageTool(
                format!("{:?}", cmd),
                format!("{}", e),
            ));
        }
    };

    if !output.status.success() {
        return Err(DfxError::PackageToolReportedError(
            format!("{:?}", cmd),
            format!("{}", output.status),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    } else if !output.stderr.is_empty() && !quiet {
        slog::warn!(logger, "{}", String::from_utf8_lossy(&output.stderr));
    }

    let package_arguments = String::from_utf8_lossy(&output.stdout)
        .split_ascii_whitespace()
        .map(String::from)
        .collect();

    Ok(package_arguments)
}
