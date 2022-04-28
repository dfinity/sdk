use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use anyhow::Context;
use clap::Parser;
use slog::info;

/// Removes an existing identity.
#[derive(Parser)]
pub struct RemoveOpts {
    /// The identity to remove.
    identity: String,
}

pub fn exec(env: &dyn Environment, opts: RemoveOpts) -> DfxResult {
    let name = opts.identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Removing identity "{}"."#, name);

    IdentityManager::new(env)
        .context("Failed to load identity manager.")?
        .remove(name)
        .context(format!("Failed to remove {}.", name))?;

    info!(log, r#"Removed identity "{}"."#, name);
    Ok(())
}
