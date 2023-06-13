use super::ExtensionManager;
use crate::lib::error::ExtensionError;
use std::path::Path;

impl ExtensionManager {
    pub fn run_extension(
        &self,
        dfx_cache: &Path,
        extension_name: String,
        mut params: Vec<String>,
    ) -> Result<(), ExtensionError> {
        let mut extension_binary = self.get_extension_binary(&extension_name)?;

        params.extend([
            "--dfx-cache-path".into(),
            dfx_cache.to_string_lossy().into(),
        ]);

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
