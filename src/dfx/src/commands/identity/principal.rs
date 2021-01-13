use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use anyhow::anyhow;
use clap::Clap;
use ic_agent::identity::Identity;

/// Shows the textual representation of the Principal associated with the current identity.
#[derive(Clap)]
pub struct GetPrincipalOpts {}

pub fn exec(env: &dyn Environment, _opts: GetPrincipalOpts) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let principal_id = identity
        .as_ref()
        .sender()
        .map_err(|err| anyhow!("{}", err))?;
    println!("{}", principal_id.to_text());
    Ok(())
}
