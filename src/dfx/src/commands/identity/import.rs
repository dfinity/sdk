use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::{IdentityCreationParameters, IdentityManager};

use clap::Parser;
use slog::info;
use std::path::PathBuf;

/// Creates a new identity from a PEM file.
#[derive(Parser)]
pub struct ImportOpts {
    /// The identity to create.
    identity: String,

    /// The PEM file to import.
    pem_file: PathBuf,

    /// DANGEROUS: By default, PEM files are encrypted with a password when writing them to disk.
    /// I you want the convenience of not having to type your password (but at the risk of having your PEM file compromised), you can disable the encryption.
    #[clap(long)]
    disable_encryption: bool,

    /// If the identity already exists, remove and re-import it.
    #[clap(long)]
    force: bool,
}

/// Executes the import subcommand.
pub fn exec(env: &dyn Environment, opts: ImportOpts) -> DfxResult {
    let log = env.get_logger();
    let name = opts.identity.as_str();
    let params = IdentityCreationParameters::PemFile {
        src_pem_file: opts.pem_file,
        disable_encryption: opts.disable_encryption,
    };
    IdentityManager::new(env)?.create_new_identity(name, params, opts.force)?;
    info!(log, r#"Imported identity: "{}"."#, name);
    Ok(())
}
