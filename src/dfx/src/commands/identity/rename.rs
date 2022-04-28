use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use anyhow::Context;
use clap::Parser;
use slog::info;

/// Renames an existing identity.
#[derive(Parser)]
pub struct RenameOpts {
    /// The current name of the identity.
    from: String,

    /// The new name of the identity.
    to: String,
}

pub fn exec(env: &dyn Environment, opts: RenameOpts) -> DfxResult {
    let from = opts.from.as_str();
    let to = opts.to.as_str();

    let log = env.get_logger();
    info!(log, r#"Renaming identity "{}" to "{}"."#, from, to);

    let mut identity_manager =
        IdentityManager::new(env).context("Failed to load identity manager.")?;
    let renamed_default = identity_manager
        .rename(env, from, to)
        .context(format!("Failed to rename {} to {}.", from, to))?;

    info!(log, r#"Renamed identity "{}" to "{}"."#, from, to);
    if renamed_default {
        info!(log, r#"Now using identity: "{}"."#, to);
    }

    Ok(())
}
