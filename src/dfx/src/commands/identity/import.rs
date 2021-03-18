use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::{IdentityCreationParameters, IdentityManager};

use clap::Clap;
use slog::info;
use std::path::PathBuf;

/// Creates a new identity from a PEM file.
#[derive(Clap)]
pub struct ImportOpts {
    /// The identity to create.
    identity: String,

    /// The PEM file to import.
    pem_file: PathBuf,
}

/// Executes the import subcommand.
pub fn exec(env: &dyn Environment, opts: ImportOpts) -> DfxResult {
    let log = env.get_logger();
    let name = opts.identity.as_str();
    info!(log, r#"Creating identity: "{}"."#, name);
    let params = IdentityCreationParameters::PemFile(opts.pem_file);
    IdentityManager::new(env)?.create_new_identity(name, params)?;
    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}
