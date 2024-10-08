use super::ExtensionManager;
use crate::config::cache::get_bin_cache;
use crate::error::extension::RunExtensionError;
use std::ffi::OsString;
use std::path::PathBuf;

impl ExtensionManager {
    pub fn run_extension(
        &self,
        extension_name: OsString,
        mut params: Vec<OsString>,
        project_root: Option<PathBuf>,
    ) -> Result<(), RunExtensionError> {
        let extension_name = extension_name
            .into_string()
            .map_err(RunExtensionError::InvalidExtensionName)?;

        let mut extension_binary = self.get_extension_binary(&extension_name)?;
        if let Some(project_root) = project_root {
            extension_binary.current_dir(project_root);
        }
        let dfx_cache = get_bin_cache(self.dfx_version.to_string().as_str())?;

        params.extend(["--dfx-cache-path".into(), dfx_cache.into_os_string()]);

        let mut child = extension_binary
            .args(&params)
            .spawn()
            .map_err(|e| RunExtensionError::FailedToLaunchExtension(extension_name.clone(), e))?;

        let exit_status = child.wait().map_err(|e| {
            RunExtensionError::ExtensionNeverFinishedExecuting(extension_name.clone(), e)
        })?;

        let code = exit_status
            .code()
            .ok_or(RunExtensionError::ExtensionExecutionTerminatedViaSignal)?;

        if code != 0 {
            Err(RunExtensionError::ExtensionExitedWithNonZeroStatus(code))
        } else {
            Ok(())
        }
    }
}
