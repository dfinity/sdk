use crate::error::foundation::FoundationError;
use crate::error::foundation::FoundationError::NoHomeInEnvironment;
use std::ffi::OsString;
use std::path::PathBuf;

pub fn get_user_home() -> Result<OsString, FoundationError> {
    std::env::var_os("HOME").ok_or(NoHomeInEnvironment())
}

pub fn get_current_exe() -> Result<PathBuf, FoundationError> {
    std::env::current_exe().map_err(FoundationError::NoCurrentExe)
}
