use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;

use clap::Parser;

/// Prints the decrypted PEM file for the identity.
#[derive(Parser)]
pub struct ExportOpts {
    /// The identity to export.
    identity: String,
}

pub fn exec(env: &dyn Environment, opts: ExportOpts) -> DfxResult {
    let name = opts.identity.as_str();

    let pem = IdentityManager::new(env)?.export(name)?;
    print!("{}", pem);

    Ok(())
}
