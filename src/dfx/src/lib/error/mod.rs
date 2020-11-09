use std::error;
use std::fmt;

pub mod build;
pub mod cache;
pub mod config;
pub mod identity;

pub use build::BuildErrorKind;
pub use cache::CacheErrorKind;
pub use config::ConfigErrorKind;
pub use identity::IdentityErrorKind;

/// The type to represent DFX results.
pub type DfxResult<T = ()> = anyhow::Result<T>;

/// The type to represent DFX errors.
pub type DfxError = anyhow::Error;

/// The type to represent legacy DFX errors.
#[derive(Debug)]
pub struct LegacyDfxError(pub String);

impl fmt::Display for LegacyDfxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for LegacyDfxError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.0)
    }
}

#[macro_export]
macro_rules! error_invalid_argument {
    ($($args:tt)*) => {
        anyhow::Error::new(
            crate::lib::error::LegacyDfxError(
                format!("Invalid argument: {}", format_args!($($args)*))
            )
        )
    }
}

#[macro_export]
macro_rules! error_invalid_data {
    ($($args:tt)*) => {
        anyhow::Error::new(
            crate::lib::error::LegacyDfxError(
                format!("Invalid data: {}", format_args!($($args)*))
            )
        )
    }
}

#[macro_export]
macro_rules! error_unknown {
    ($($args:tt)*) => {
        anyhow::Error::new(
            crate::lib::error::LegacyDfxError(
                format!("Unknown error: {}", format_args!($($args)*))
            )
        )
    }
}
