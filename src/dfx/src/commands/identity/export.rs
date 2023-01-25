use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

/// Prints the decrypted PEM file for the identity.
#[derive(Parser)]
pub struct ExportOpts {
    /// The identity to export.
    exported_identity: String,
}

pub fn exec(env: &dyn Environment, opts: ExportOpts) -> DfxResult {
    let name = opts.exported_identity.as_str();

    let pem = env.new_identity_manager()?.export(env.get_logger(), name)?;
    print!("{}", pem);

    Ok(())
}
