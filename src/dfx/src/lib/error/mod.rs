pub mod build;
pub mod cache;
pub mod extension;

pub use build::BuildError;
pub use cache::CacheError;
pub use dfx_core::error::identity::IdentityError;
pub use extension::ExtensionError;

/// The type to represent DFX results.
pub type DfxResult<T = ()> = anyhow::Result<T>;

/// The type to represent DFX errors.
pub type DfxError = anyhow::Error;

#[macro_export]
macro_rules! error_invalid_argument {
    ($($args:tt)*) => {
        anyhow::anyhow!("Invalid argument: {}", format_args!($($args)*))
    }
}

#[macro_export]
macro_rules! error_invalid_config {
    ($($args:tt)*) => {
        anyhow::anyhow!("Invalid configuration: {}", format_args!($($args)*))
    }
}

#[macro_export]
macro_rules! error_invalid_data {
    ($($args:tt)*) => {
        anyhow::anyhow!("Invalid data: {}", format_args!($($args)*))
    }
}

#[macro_export]
macro_rules! error_unknown {
    ($($args:tt)*) => {
        anyhow::anyhow!("Unknown error: {}", format_args!($($args)*))
    }
}
