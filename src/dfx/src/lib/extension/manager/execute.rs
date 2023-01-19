use super::ExtensionsManager;
use crate::lib::error::DfxResult;
use std::ffi::OsString;

// possible errors:
// - no such extension
// - extension is not installed
// - insufficient permissions to execute the extension
// - extension failed to execute

impl ExtensionsManager {
    pub fn run_extension(&self, extension_name: OsString, params: Vec<OsString>) -> DfxResult<()> {
        if let Ok(mut extension_binary) =
            self.get_extension_binary(&extension_name.to_string_lossy())
        {
            return extension_binary
                .args(&params)
                .spawn()
                .expect("failed to execute process")
                .wait()
                .expect("failed to wait on child")
                .code()
                .map_or(Ok(()), |code| {
                    if code != 0 {
                        Err(anyhow::anyhow!("Extension exited with code {}", code))
                    } else {
                        Ok(())
                    }
                });
        } else {
            Err(anyhow::anyhow!(
                "extension {:?} does cannot be found",
                extension_name
            ))
        }
    }
}
