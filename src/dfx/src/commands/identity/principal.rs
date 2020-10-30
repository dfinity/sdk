use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use clap::{App, ArgMatches, Clap, IntoApp};
use ic_agent::Identity;

/// Shows the textual representation of the Principal associated with the current identity.
#[derive(Clap)]
pub struct GetPrincipalOpts {}

pub fn construct() -> App<'static> {
    GetPrincipalOpts::into_app().name("get-principal")
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let principal_id = identity.as_ref().sender()?;
    println!("{}", principal_id.to_text());
    Ok(())
}
