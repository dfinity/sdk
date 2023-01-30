use super::ExtensionManager;
use crate::lib::error::ExtensionError;
use std::ffi::OsString;

impl ExtensionManager {
    pub fn run_extension(
        &self,
        extension_name: OsString,
        params: Vec<OsString>,
    ) -> Result<(), ExtensionError> {
        let Ok(extension_name) = extension_name.clone().into_string() else {
            return Err(ExtensionError::InvalidExtensionName(extension_name.to_string_lossy().to_string()))
        };
        let mut extension_binary = self.get_extension_binary(&extension_name)?;

        let Ok(mut child) = extension_binary.args(&params).spawn() else {
            return Err(ExtensionError::FailedToLaunchExtension(extension_name))
        };

        let Ok(exit_status) = child.wait() else {
            return Err(ExtensionError::ExtensionNeverFinishedExecuting(extension_name))
        };

        let Some(code) = exit_status.code() else {
            #[cfg(not(target_os = "windows"))]
            return Err(ExtensionError::ExtensionExecutionTerminatedViaSignal)
        };

        if code != 0 {
            Err(ExtensionError::ExtensionExitedWithNonZeroStatus(code))
        } else {
            Ok(())
        }
    }
}
