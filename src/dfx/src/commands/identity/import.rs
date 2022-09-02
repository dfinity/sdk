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

    /// DANGEROUS: By default, PEM files are encrypted with a password when writing them to disk.
    /// If you want the convenience of not having to type your password (but at the risk of having your PEM file compromised), you can disable the encryption.
    #[clap(long)]
    disable_encryption: bool,

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
            src_pem_file,
            disable_encryption: opts.disable_encryption,
        }
    } else {
        let mnemonic =
            fs::read_to_string(opts.seed_file.unwrap()).context("Failed to read seed file")?;
        IdentityCreationParameters::SeedPhrase {
            mnemonic,
            disable_encryption: opts.disable_encryption,
        }
    };
    IdentityManager::new(env)?.create_new_identity(name, params, opts.force)?;
    info!(log, r#"Imported identity: "{}"."#, name);
    Ok(())
}
