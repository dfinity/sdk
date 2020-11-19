use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use clap::Clap;
use ic_agent::Identity;

/// Shows the textual representation of the Principal associated with the current identity.
#[derive(Clap)]
#[clap(name("get-principal"))]
pub struct GetPrincipalOpts {}

pub fn exec(env: &dyn Environment, _opts: GetPrincipalOpts) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let principal_id = identity.as_ref().sender()?;
    println!("{}", principal_id.to_text());
    Ok(())
}
