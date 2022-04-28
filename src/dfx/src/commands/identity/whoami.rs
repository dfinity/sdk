use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use anyhow::Context;
use clap::Parser;

/// Shows the name of the current identity.
#[derive(Parser)]
pub struct WhoAmIOpts {}

pub fn exec(env: &dyn Environment, _opts: WhoAmIOpts) -> DfxResult {
    let mgr = IdentityManager::new(env).context("Failed to load identity manager.")?;
    let identity = mgr.get_selected_identity_name();
    println!("{}", identity);
    Ok(())
}
