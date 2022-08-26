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
    new_identity: String,

    /// The PEM file to import.
    pem_file: PathBuf,

    /// By default, PEM files are saved in the system's keyring.
    /// If you do not want to use your system's keyring, use this flag to have the PEM file saved in encrypted format on disk.
    /// This will require you to enter the password for almost every dfx command.
    #[clap(long)]
    skip_keyring: bool,

    /// If the identity already exists, remove and re-import it.
    #[clap(long)]
    force: bool,
}

/// Executes the import subcommand.
pub fn exec(env: &dyn Environment, opts: ImportOpts) -> DfxResult {
    let log = env.get_logger();
    let name = opts.new_identity.as_str();
    let params = IdentityCreationParameters::PemFile {
        src_pem_file: opts.pem_file,
        skip_keyring: opts.skip_keyring,
    };
    IdentityManager::new(env)?.create_new_identity(name, params, opts.force)?;
    info!(log, r#"Imported identity: "{}"."#, name);
    Ok(())
}
