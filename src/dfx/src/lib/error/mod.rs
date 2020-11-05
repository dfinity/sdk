mod build;
mod cache;
mod config;
mod identity;

pub use build::BuildErrorKind;
pub use cache::CacheErrorKind;
pub use config::ConfigErrorKind;
pub use identity::IdentityErrorKind;

use anyhow;
use mime;

/// The type to represent DFX failures.
pub type DfxError = anyhow::Error;

/// The result of running a DFX command.
pub type DfxResult<T = ()> = anyhow::Result<T>;

fn is_plain_text_utf8(content_type: &Option<String>) -> bool {
    // text/plain is also sometimes returned by the replica (or ic-ref),
    // depending on where in the stack the error happens.
    matches!(
        content_type.as_ref().and_then(|s|s.parse::<mime::Mime>().ok()),
        Some(mt) if mt == mime::TEXT_PLAIN || mt == mime::TEXT_PLAIN_UTF_8
    )
}
