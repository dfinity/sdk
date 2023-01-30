use super::ExtensionManager;
use crate::lib::error::ExtensionError;
use std::ffi::OsString;

impl ExtensionManager {
    pub fn run_extension(
        &self,
        extension_name: OsString,
        params: Vec<OsString>,
    ) -> Result<(), ExtensionError> {
        let extension_name = extension_name.clone().into_string().map_err(|_e| {
            ExtensionError::InvalidExtensionName(extension_name.to_string_lossy().to_string())
        })?;

        let mut extension_binary = self.get_extension_binary(&extension_name)?;

        let mut child = extension_binary
            .args(&params)
            .spawn()
            .map_err(|_e| ExtensionError::FailedToLaunchExtension(extension_name.clone()))?;

        let exit_status = child.wait().map_err(|_e| {
            ExtensionError::ExtensionNeverFinishedExecuting(extension_name.clone())
        })?;

        let code = exit_status.code().ok_or(
            #[cfg(not(target_os = "windows"))]
            ExtensionError::ExtensionExecutionTerminatedViaSignal,
        )?;

        if code != 0 {
            Err(ExtensionError::ExtensionExitedWithNonZeroStatus(code))
        } else {
            Ok(())
        }
    }
}
