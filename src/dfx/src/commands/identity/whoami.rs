use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

/// Shows the name of the current identity.
#[derive(Parser)]
pub struct WhoAmIOpts {}

pub fn exec(env: &dyn Environment, _opts: WhoAmIOpts) -> DfxResult {
    let mgr = env.new_identity_manager()?;
    let identity = mgr.get_selected_identity_name();
    println!("{}", identity);
    Ok(())
}
