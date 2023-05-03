use crate::commands::identity::new::create_new_dfx_identity;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use dfx_core::identity::identity_manager::{IdentityCreationParameters, IdentityStorageMode};

use anyhow::Context;
use clap::Parser;
use slog::{info, warn};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// Creates a new identity from a PEM file.
#[derive(Parser)]
pub struct ImportOpts {
    /// The identity to create.
    new_identity: String,

    /// The PEM file to import.
    pem_file: Option<PathBuf>,

    /// The path to a file with your seed phrase.
    #[arg(long, conflicts_with("pem_file"), required_unless_present("pem_file"))]
    seed_file: Option<PathBuf>,

    /// DEPRECATED: Please use --storage-mode=plaintext instead
    #[arg(long)]
    disable_encryption: bool,

    /// How your private keys are stored. By default, if keyring/keychain is available, keys are stored there.
    /// Otherwise, a password-protected file is used as fallback.
    /// Mode 'plaintext' is not safe, but convenient for use in CI.
    #[arg(long, conflicts_with("disable_encryption"),
        value_parser = ["keyring", "password-protected", "plaintext"])]
    storage_mode: Option<String>,

    /// If the identity already exists, remove and re-import it.
    #[arg(long)]
    force: bool,
}

/// Executes the import subcommand.
pub fn exec(env: &dyn Environment, opts: ImportOpts) -> DfxResult {
    let log = env.get_logger();

    if opts.disable_encryption {
        warn!(log, "The flag --disable-encryption has been deprecated. Please use --storage-mode=plaintext instead.");
    }

    let mode = if opts.disable_encryption {
        IdentityStorageMode::Plaintext
    } else if let Some(mode_str) = opts.storage_mode {
        IdentityStorageMode::from_str(&mode_str)?
    } else {
        IdentityStorageMode::default()
    };
    let name = opts.new_identity.as_str();
    let params = if let Some(src_pem_file) = opts.pem_file {
        IdentityCreationParameters::PemFile { src_pem_file, mode }
    } else {
        let mnemonic =
            fs::read_to_string(opts.seed_file.unwrap()).context("Failed to read seed file")?;
        IdentityCreationParameters::SeedPhrase { mnemonic, mode }
    };

    create_new_dfx_identity(env, log, name, params, opts.force)?;

    info!(log, r#"Imported identity: "{}"."#, name);
    Ok(())
}
