use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use clap::Clap;
use slog::info;

/// Removes an existing identity.
#[derive(Clap)]
#[clap(name("remove"))]
pub struct RemoveOpts {
    /// The identity to remove.
    identity: String,
}

pub fn exec(env: &dyn Environment, opts: RemoveOpts) -> DfxResult {
    let name = opts.identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Removing identity "{}"."#, name);

    IdentityManager::new(env)?.remove(name)?;

    info!(log, r#"Removed identity "{}"."#, name);
    Ok(())
}
