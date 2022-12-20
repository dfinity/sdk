use crate::error::foundation::FoundationError;
use crate::error::foundation::FoundationError::NoHomeInEnvironment;

pub fn get_user_home() -> Result<String, FoundationError> {
    std::env::var("HOME").map_err(|_| NoHomeInEnvironment())
}
