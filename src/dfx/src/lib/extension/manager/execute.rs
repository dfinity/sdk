use super::ExtensionManager;
use crate::lib::error::{DfxError, DfxResult, ExtensionError};
use std::ffi::OsString;

impl ExtensionManager {
    pub fn run_extension(&self, extension_name: OsString, params: Vec<OsString>) -> DfxResult<()> {
        let Ok(extension_name) = extension_name.clone().into_string() else {
            return Err(DfxError::new(ExtensionError::InvalidExtensionName(extension_name.to_string_lossy().to_string())))
        };
        let mut extension_binary = self.get_extension_binary(&extension_name)?;

        let Ok(mut child) = extension_binary.args(&params).spawn() else {
            return Err(DfxError::new(ExtensionError::FailedToLaunchExtension(extension_name)))
        };

        let Ok(exit_status) = child.wait() else {
            return Err(DfxError::new(ExtensionError::ExtensionNeverFinishedExecuting(extension_name)))
        };

        let Some(code) = exit_status.code() else {
            #[cfg(not(target_os = "windows"))]
            return Err(DfxError::new(ExtensionError::ExtensionExecutionTerminatedViaSignal))
        };

        if code != 0 {
            Err(DfxError::new(
                ExtensionError::ExtensionExitedWithNonZeroStatus(code),
            ))
        } else {
            Ok(())
        }
    }
}
