use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::{IdentityCreationParameters, IdentityManager};

use anyhow::Context;
use clap::Parser;
use slog::info;
use std::fs;
use std::path::PathBuf;

/// Creates a new identity from a PEM file.
#[derive(Parser)]
pub struct ImportOpts {
    /// The identity to create.
    new_identity: String,

    /// The PEM file to import.
    pem_file: Option<PathBuf>,

    /// The path to a file with your seed phrase.
    #[clap(long, conflicts_with("pem-file"), required_unless_present("pem-file"))]
    seed_file: Option<PathBuf>,

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
    let params = if let Some(src_pem_file) = opts.pem_file {
        IdentityCreationParameters::PemFile {
            skip_keyring: opts.skip_keyring,
            src_pem_file,
        }
    } else {
        let mnemonic =
            fs::read_to_string(opts.seed_file.unwrap()).context("Failed to read seed file")?;
        IdentityCreationParameters::SeedPhrase {
            mnemonic,
            skip_keyring: opts.skip_keyring,
        }
    };
    IdentityManager::new(env)?.create_new_identity(log, name, params, opts.force)?;
    info!(log, r#"Imported identity: "{}"."#, name);
    Ok(())
}
