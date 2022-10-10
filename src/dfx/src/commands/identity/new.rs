use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::{
    HardwareIdentityConfiguration, IdentityCreationParameters, IdentityManager,
};
use crate::util::clap::validators::is_hsm_key_id;

use clap::Parser;
use slog::info;
use IdentityCreationParameters::{Hardware, Pem};

/// Creates a new identity.
#[derive(Parser)]
pub struct NewIdentityOpts {
    /// The identity to create.
    new_identity: String,

    /// The file path to the opensc-pkcs11 library e.g. "/usr/local/lib/opensc-pkcs11.so"
    #[clap(long, requires("hsm-key-id"))]
    hsm_pkcs11_lib_path: Option<String>,

    /// A sequence of pairs of hex digits
    #[clap(long, requires("hsm-pkcs11-lib-path"), validator(is_hsm_key_id))]
    hsm_key_id: Option<String>,

    /// DANGEROUS: By default, PEM files are encrypted with a password when writing them to disk.
    /// If you want the convenience of not having to type your password (but at the risk of having your PEM file compromised), you can disable the encryption.
    #[clap(long)]
    disable_encryption: bool,

    /// If the identity already exists, remove and re-create it.
    #[clap(long)]
    force: bool,
}

pub fn exec(env: &dyn Environment, opts: NewIdentityOpts) -> DfxResult {
    let name = opts.new_identity.as_str();

    let log = env.get_logger();

    let creation_parameters = match (opts.hsm_pkcs11_lib_path, opts.hsm_key_id) {
        (Some(pkcs11_lib_path), Some(key_id)) => Hardware {
            hsm: HardwareIdentityConfiguration {
                pkcs11_lib_path,
                key_id,
            },
        },
        _ => Pem {
            disable_encryption: opts.disable_encryption,
        },
    };

    IdentityManager::new(env)?.create_new_identity(name, creation_parameters, opts.force)?;

    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}
