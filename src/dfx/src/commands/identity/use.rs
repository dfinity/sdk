use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use clap::{App, ArgMatches, Clap, FromArgMatches, IntoApp};
use slog::info;

/// Specifies the identity to use.
#[derive(Clap)]
#[clap(name("use"))]
pub struct UseOpts {
    /// The identity to use.
    identity: String,
}

pub fn construct() -> App<'static> {
    UseOpts::into_app()
}

pub fn exec(env: &dyn Environment, args: &ArgMatches) -> DfxResult {
    let opts: UseOpts = UseOpts::from_arg_matches(args);
    let identity = opts.identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Using identity: "{}"."#, identity);

    IdentityManager::new(env)?.use_identity_named(identity)
}
