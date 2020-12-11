use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::{
    HardwareIdentityConfiguration, IdentityCreationParameters, IdentityManager,
};

use clap::Clap;
use slog::info;
use IdentityCreationParameters::{Hardware, Pem};

/// Creates a new identity.
#[derive(Clap)]
#[clap(name("new"))]
pub struct NewIdentityOpts {
    /// The identity to create.
    identity: String,

    /// The file path to the opensc-pkcs11 library e.g. "/usr/local/lib/opensc-pkcs11.so"
    #[clap(long, requires("hsm-key-id"))]
    hsm_pkcs11_lib_path: Option<String>,

    /// A sequence of pairs of hex digits
    #[clap(long, requires("hsm-pkcs11-lib-path"), validator(is_key_id))]
    hsm_key_id: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: NewIdentityOpts) -> DfxResult {
    let name = opts.identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Creating identity: "{}"."#, name);

    let creation_parameters = match (opts.hsm_pkcs11_lib_path, opts.hsm_key_id) {
        (Some(pkcs11_lib_path), Some(key_id)) => Hardware(HardwareIdentityConfiguration {
            pkcs11_lib_path,
            key_id,
        }),
        _ => Pem(),
    };

    IdentityManager::new(env)?.create_new_identity(name, creation_parameters)?;

    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}

fn is_key_id(key_id: &str) -> Result<(), String> {
    if key_id.len() % 2 != 0 {
        Err("Key id must consist of an even number of hex digits".to_string())
    } else if key_id.contains(|c: char| !c.is_ascii_hexdigit()) {
        Err("Key id must contain only hex digits".to_string())
    } else {
        Ok(())
    }
}
