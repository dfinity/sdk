use crate::config::cache::Cache;
use crate::lib::error::{BuildError, DfxError, DfxResult};

use anyhow::anyhow;
use std::process::Command;

/// Package arguments for moc or mo-ide as returned by
/// a package tool like https://github.com/kritzcreek/vessel
/// or, if there is no package tool, the base library.
pub type PackageArguments = Vec<String>;

pub fn load(cache: &dyn Cache, packtool: &Option<String>) -> DfxResult<PackageArguments> {
    if packtool.is_none() {
        let stdlib_path = cache
            .get_binary_command_path("base")?
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow!("Path contains invalid Unicode data."))?;
        let base = vec![String::from("--package"), String::from("base"), stdlib_path];
        return Ok(base);
    }

    let commandline: Vec<String> = packtool
        .as_ref()
        .unwrap()
        .split_ascii_whitespace()
        .map(String::from)
        .collect();

    let mut cmd = Command::new(commandline[0].clone());
    for arg in commandline.iter().skip(1) {
        cmd.arg(arg);
    }

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(DfxError::new(BuildError::CommandError(
            format!("{:?}", cmd),
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )));
    }

    let package_arguments = String::from_utf8_lossy(&output.stdout)
        .split_ascii_whitespace()
        .map(String::from)
        .collect();

    Ok(package_arguments)
}
