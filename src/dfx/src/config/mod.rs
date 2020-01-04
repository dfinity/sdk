pub mod cache;
pub mod dfinity;

/// Returns the version of DFX that was built.
/// In debug, add a timestamp of the upstream compilation at the end of version to ensure all
/// debug runs are unique (and cached uniquely).
/// That timestamp is taken from the DFX_TIMESTAMP_DEBUG_MODE_ONLY env var that is set in
/// Nix.
pub fn dfx_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
