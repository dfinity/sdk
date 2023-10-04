use crate::error::get_current_exe::GetCurrentExeError;
use crate::error::get_current_exe::GetCurrentExeError::NoCurrentExe;
use crate::error::get_user_home::GetUserHomeError;
use crate::error::get_user_home::GetUserHomeError::NoHomeInEnvironment;
use std::ffi::OsString;
use std::path::PathBuf;

pub fn get_user_home() -> Result<OsString, GetUserHomeError> {
    std::env::var_os("HOME").ok_or(NoHomeInEnvironment())
}

pub fn get_current_exe() -> Result<PathBuf, GetCurrentExeError> {
    std::env::current_exe().map_err(NoCurrentExe)
}
