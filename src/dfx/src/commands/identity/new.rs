use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use slog::info;

/// Creates a new identity.
#[derive(Clap)]
#[clap(name("new"))]
pub struct NewIdentityOpts {
    /// The identity to create.
    identity: String,
}

pub fn construct() -> App<'static> {
    NewIdentityOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: NewIdentityOpts = NewIdentityOpts::from_arg_matches(args);
    let name = opts.identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Creating identity: "{}"."#, name);

    IdentityManager::new(env)?.create_new_identity(name)?;

    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}
