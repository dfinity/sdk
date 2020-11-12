use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use slog::info;

/// Removes an existing identity.
#[derive(Clap)]
#[clap(name("remove"))]
pub struct RemoveOpts {
    /// The identity to remove.
    identity: String,
}

pub fn construct() -> App<'static> {
    RemoveOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: RemoveOpts = RemoveOpts::from_arg_matches(args);
    let name = opts.identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Removing identity "{}"."#, name);

    IdentityManager::new(env)?.remove(name)?;

    info!(log, r#"Removed identity "{}"."#, name);
    Ok(())
}
