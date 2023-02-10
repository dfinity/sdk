use super::ExtensionManager;
use crate::lib::error::ExtensionError;
use std::ffi::OsString;

impl ExtensionManager {
    pub fn run_extension(
        &self,
        extension_name: OsString,
        params: Vec<OsString>,
    ) -> Result<(), ExtensionError> {
        let extension_name = extension_name
            .into_string()
            .map_err(ExtensionError::InvalidExtensionName)?;

        let mut extension_binary = self.get_extension_binary(&extension_name)?;

        let mut child = extension_binary
            .args(&params)
            .spawn()
            .map_err(|e| ExtensionError::FailedToLaunchExtension(extension_name.clone(), e))?;

        let exit_status = child.wait().map_err(|e| {
            ExtensionError::ExtensionNeverFinishedExecuting(extension_name.clone(), e)
        })?;

        let code = exit_status
            .code()
            .ok_or(ExtensionError::ExtensionExecutionTerminatedViaSignal)?;

        if code != 0 {
            Err(ExtensionError::ExtensionExitedWithNonZeroStatus(code))
        } else {
            Ok(())
        }
    }
}
