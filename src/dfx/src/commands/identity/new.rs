use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::{
    HardwareSecurityModuleConfiguration, IdentityCreationParameters, IdentityManager,
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

    /// A sequence of pairs of hex digits
    // todo validator
    #[clap(long, requires("hsm_key_id"))]
    hsm_filename: Option<String>,

    /// Something like "/usr/local/lib/opensc-pkcs11.so"
    #[clap(long, requires("hsm_filename"))]
    hsm_key_id: Option<String>,
}

pub fn exec(env: &dyn Environment, opts: NewIdentityOpts) -> DfxResult {
    let name = opts.identity.as_str();

    let log = env.get_logger();
    info!(log, r#"Creating identity: "{}"."#, name);

    let creation_parameters = match (opts.hsm_filename, opts.hsm_key_id) {
        (Some(filename), Some(key_id)) => {
            Hardware(HardwareSecurityModuleConfiguration { filename, key_id })
        }
        _ => Pem(),
    };

    IdentityManager::new(env)?.create_new_identity(name, creation_parameters)?;

    info!(log, r#"Created identity: "{}"."#, name);
    Ok(())
}
