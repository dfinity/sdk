pub mod convert_mnemonic_to_key;
pub mod create_identity_config;
pub mod create_new_identity;
pub mod export_identity;
pub mod generate_key;
pub mod get_identity_config_or_default;
pub mod get_legacy_credentials_pem_path;
pub mod initialize_identity_manager;
pub mod instantiate_identity_from_name;
pub mod load_identity;
pub mod load_pem;
pub mod load_pem_from_file;
pub mod load_pem_identity;
pub mod map_wallets_to_renamed_identity;
pub mod new_hardware_identity;
pub mod new_identity;
pub mod new_identity_manager;
pub mod remove_identity;
pub mod rename_identity;
pub mod rename_wallet_global_config_key;
pub mod save_identity_configuration;
pub mod save_pem;
pub mod use_identity_by_name;
pub mod validate_pem_file;
pub mod write_default_identity;
pub mod write_pem_to_file;

use ic_agent::export::PrincipalError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Identity {0} does not exist at '{1}'.")]
    IdentityDoesNotExist(String, PathBuf),

    #[error("Failed to read principal from id '{0}': {1}")]
    ParsePrincipalFromIdFailed(String, PrincipalError),

    #[error("An Identity named {0} cannot be created as it is reserved for internal use.")]
    ReservedIdentityName(String),
}
