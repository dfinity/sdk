use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

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

    IdentityManager::new(env)?.use_identity_named(identity)
}
