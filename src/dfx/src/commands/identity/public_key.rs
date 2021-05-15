use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use anyhow::anyhow;
use clap::Clap;

/// Shows the public key associated with the current identity, in HEX-encode DER.
#[derive(Clap)]
pub struct GetPublicKeyOpts {}

pub fn exec(env: &dyn Environment, _opts: GetPublicKeyOpts) -> DfxResult {
    let identity = IdentityManager::new(env)?.instantiate_selected_identity()?;
    let public_key = identity
        .as_ref()
        .public_key()
        .map_err(|err| anyhow!("{}", err))?;
    println!("{}", hex::encode(public_key));
    Ok(())
}
