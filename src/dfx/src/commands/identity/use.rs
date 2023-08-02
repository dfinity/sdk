use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use clap::Parser;
use slog::info;

/// Specifies the identity to use.
#[derive(Parser)]
pub struct UseOpts {
    /// The identity to use.
    new_identity: String,
}

pub fn exec(env: &dyn Environment, opts: UseOpts) -> DfxResult {
    let identity = opts.new_identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Using identity: "{}"."#, identity);

    env.new_identity_manager()?
        .use_identity_named(log, identity)
        .map_err(DfxError::new)
}
