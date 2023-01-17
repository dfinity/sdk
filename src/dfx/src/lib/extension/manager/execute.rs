use super::ExtensionsManager;
use crate::lib::error::DfxResult;
use std::ffi::OsString;

impl ExtensionsManager {
    pub fn run_extension(&self, extension_name: OsString, params: Vec<OsString>) -> DfxResult<()> {
        if let Ok(mut extension_binary) =
            self.get_extension_binary(extension_name.to_str().unwrap())
        {
            return extension_binary
                .args(&params)
                .spawn()
                .expect("failed to execute process")
                .wait()
                .expect("failed to wait on child")
                .code()
                .map_or(Ok(()), |code| {
                    Err(anyhow::anyhow!("Extension exited with code {}", code))
                });
        } else {
            Err(anyhow::anyhow!(
                "extension {:?} does cannot be found",
                extension_name
            ))
        }
    }
}
