use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use anyhow::anyhow;
use clap::{App, ArgMatches, Clap, IntoApp};
use ic_agent::Identity;

/// Shows the textual representation of the Principal associated with the current identity.
#[derive(Clap)]
#[clap(name("get-principal"))]
pub struct GetPrincipalOpts {}

pub fn construct() -> App<'static> {
    GetPrincipalOpts::into_app()
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let principal_id = identity
        .as_ref()
        .sender()
        .map_err(|err| anyhow!("{}", err))?;
    println!("{}", principal_id.to_text());
    Ok(())
}
