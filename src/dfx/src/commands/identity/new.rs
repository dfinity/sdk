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

    /// By default, PEM files are saved in the system's keyring.
    /// If you do not want to use your system's keyring, use this flag to have the PEM file saved in encrypted format on disk.
    /// This will require you to enter the password for almost every dfx command.
    #[clap(long)]
    skip_keyring: bool,

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
            skip_keyring: opts.skip_keyring,
        },
    };

    IdentityManager::new(env)?.create_new_identity(name, creation_parameters, opts.force)?;

    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}
