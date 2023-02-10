use crate::error::foundation::FoundationError;
use crate::error::foundation::FoundationError::NoHomeInEnvironment;
use std::ffi::OsString;

pub fn get_user_home() -> Result<OsString, FoundationError> {
    std::env::var_os("HOME").ok_or(NoHomeInEnvironment())
}
