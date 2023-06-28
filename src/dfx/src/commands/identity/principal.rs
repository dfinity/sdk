use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use anyhow::anyhow;
use clap::Parser;
use ic_agent::identity::Identity;

/// Shows the textual representation of the Principal associated with the current identity.
#[derive(Parser)]
pub struct GetPrincipalOpts {}

pub fn exec(env: &dyn Environment, _opts: GetPrincipalOpts) -> DfxResult {
    let identity = env
        .new_identity_manager()?
        .instantiate_selected_identity(env.get_logger())?;
    let principal_id = identity
        .as_ref()
        .sender()
        .map_err(|err| anyhow!("{}", err))?;
    println!("{}", principal_id.to_text());
    Ok(())
}
