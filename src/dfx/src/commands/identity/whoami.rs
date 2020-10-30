use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use clap::{App, ArgMatches, Clap, IntoApp};

/// Shows the name of the current identity.
#[derive(Clap)]
pub struct WhoAmIOpts {}

pub fn construct() -> App<'static> {
    WhoAmIOpts::into_app().name("whoami")
}

pub fn exec(env: &dyn Environment, _args: &ArgMatches) -> DfxResult {
    let mgr = IdentityManager::new(env)?;
    let identity = mgr.get_selected_identity_name();
    println!("{}", identity);
    Ok(())
}
