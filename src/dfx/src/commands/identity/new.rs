use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::clap::parsers::hsm_key_id_parser;
use anyhow::Context;
use clap::Parser;
use dfx_core::error::identity::create_new_identity::CreateNewIdentityError::SwitchBackToIdentityFailed;
use dfx_core::identity::identity_manager::{
    HardwareIdentityConfiguration, IdentityCreationParameters, IdentityStorageMode,
};
use regex::Regex;
use slog::{info, warn, Logger};
use std::str::FromStr;
use IdentityCreationParameters::{Hardware, Pem};

/// Creates a new identity.
#[derive(Parser)]
pub struct NewIdentityOpts {
    #[arg(value_parser = identity_name_validator)]
    /// The name of the identity to create. Valid characters are letters, numbers, and these symbols: .-_@
    new_identity: String,

    #[cfg_attr(
        not(windows),
        doc = r#"The file path to the opensc-pkcs11 library e.g. "/usr/local/lib/opensc-pkcs11.so""#
    )]
    #[cfg_attr(
        windows,
        doc = r#"The file path to the opensc-pkcs11 library e.g. "C:\Program Files (x86)\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll"#
    )]
    #[arg(long, requires("hsm_key_id"))]
    hsm_pkcs11_lib_path: Option<String>,

    /// A sequence of pairs of hex digits
    #[arg(long, requires("hsm_pkcs11_lib_path"), value_parser = hsm_key_id_parser)]
    hsm_key_id: Option<String>,

    /// DEPRECATED: Please use --storage-mode=plaintext instead
    #[arg(long)]
    disable_encryption: bool,

    /// How your private keys are stored. By default, if keyring/keychain is available, keys are stored there.
    /// Otherwise, a password-protected file is used as fallback.
    /// Mode 'plaintext' is not safe, but convenient for use in CI.
    #[arg(long, conflicts_with("disable_encryption"),
        value_parser = ["keyring", "password-protected", "plaintext"])]
    storage_mode: Option<String>,

    /// If the identity already exists, remove and re-create it.
    #[arg(long)]
    force: bool,
}

fn identity_name_validator(name: &str) -> Result<String, String> {
    let valid_name = Regex::new(r"^[A-Za-z0-9\.\-_@]+$").unwrap();
    if !valid_name.is_match(name) {
        return Err("Invalid identity name. Please only use the characters ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.-_@0123456789".to_string());
    }
    Ok(name.into())
}

pub fn exec(env: &dyn Environment, opts: NewIdentityOpts) -> DfxResult {
    let log = env.get_logger();

    if opts.disable_encryption {
        warn!(log, "The flag --disable-encryption has been deprecated. Please use --storage-mode=plaintext instead.");
    }

    let name = opts.new_identity.as_str();

    let creation_parameters = match (opts.hsm_pkcs11_lib_path, opts.hsm_key_id) {
        (Some(pkcs11_lib_path), Some(key_id)) => Hardware {
            hsm: HardwareIdentityConfiguration {
                pkcs11_lib_path,
                key_id,
            },
        },
        _ => {
            let mode = if opts.disable_encryption {
                IdentityStorageMode::Plaintext
            } else if let Some(mode_str) = opts.storage_mode {
                IdentityStorageMode::from_str(&mode_str)?
            } else {
                IdentityStorageMode::default()
            };

            Pem { mode }
        }
    };

    create_new_dfx_identity(env, log, name, creation_parameters, opts.force)?;

    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}

pub fn create_new_dfx_identity(
    env: &dyn Environment,
    log: &Logger,
    name: &str,
    creation_parameters: IdentityCreationParameters,
    force: bool,
) -> DfxResult {
    let result =
        env.new_identity_manager()?
            .create_new_identity(log, name, creation_parameters, force);
    if let Err(SwitchBackToIdentityFailed(underlying)) = result {
        Err(underlying).with_context(||format!("Failed to switch back over to the identity you're replacing. Please run 'dfx identity use {}' to do it manually.", name))?;
    } else {
        result?;
    }
    Ok(())
}
