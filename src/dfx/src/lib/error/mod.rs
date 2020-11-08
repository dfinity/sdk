use anyhow;

/// The type to represent DFX results.
pub type DfxResult<T = ()> = anyhow::Result<T>;

/// The type to represent DFX failures.
pub type DfxError = anyhow::Error;
