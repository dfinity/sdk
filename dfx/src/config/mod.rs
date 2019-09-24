pub mod cache;
pub mod dfinity;

static mut DFX_VERSION: Option<String> = None;
/// Returns the version of DFX that was built.
/// In debug, add a timestamp of the upstream compilation at the end of version to ensure all
/// debug runs are unique (and cached uniquely).
/// That timestamp is taken from the DFX_TIMESTAMP_DEBUG_MODE_ONLY env var that is set in
/// Nix.
pub fn dfx_version() -> &'static str {
    unsafe {
        match &DFX_VERSION {
            Some(x) => x.as_str(),
            None => {
                let version = env!("CARGO_PKG_VERSION");
                DFX_VERSION = Some(version.to_owned());

                #[cfg(debug_assertions)]
                {
                    DFX_VERSION = Some(format!(
                        "{}-{}",
                        version,
                        std::env::var("DFX_TIMESTAMP_DEBUG_MODE_ONLY")
                            .unwrap_or_else(|_| "local-debug".to_owned())
                    ));
                }

                dfx_version()
            }
        }
    }
}
