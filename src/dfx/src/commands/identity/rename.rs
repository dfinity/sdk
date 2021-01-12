use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use clap::Clap;
use slog::info;

/// Renames an existing identity.
#[derive(Clap)]
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

    let renamed_default = IdentityManager::new(env)?.rename(from, to)?;

    info!(log, r#"Renamed identity "{}" to "{}"."#, from, to);
    if renamed_default {
        info!(log, r#"Now using identity: "{}"."#, to);
    }

    Ok(())
}
