use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use clap::Clap;

/// Shows the name of the current identity.
#[derive(Clap)]
#[clap(name("whoami"))]
pub struct WhoAmIOpts {}

pub fn exec(env: &dyn Environment) -> DfxResult {
    let mgr = IdentityManager::new(env)?;
    let identity = mgr.get_selected_identity_name();
    println!("{}", identity);
    Ok(())
}
