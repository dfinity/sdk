use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanisterStoreIdError {
    #[error(transparent)]
    Io(#[from] dfx_core::error::io::IoError),

    #[error(transparent)]
    StructuredFile(#[from] dfx_core::error::structured_file::StructuredFileError),

    #[error(
        "Cannot find canister id. Please issue 'dfx canister create {canister_name}{network}'."
    )]
    CanisterIdNotFound {
        canister_name: String,
        network: String,
    },

    #[error("Failed to load canister id store: {source}")]
    LoadCanisterIdStore { source: Box<Self> },

    #[error("Failed to load canister id store for network '{network_descriptor_name}': {source}")]
    LoadCanisterIdStoreForNetwork {
        network_descriptor_name: String,
        source: Box<Self>,
    },

    #[error("Failed to remove canister '{canister_name}' from id store: {source}")]
    RemoveCanisterId {
        canister_name: String,
        source: Box<Self>,
    },

    #[error("Failed to add canister with name '{canister_name}' and id '{canister_id}' to canister id store: {source}")]
    AddCanisterId {
        canister_name: String,
        canister_id: String,
        source: Box<Self>,
    },
}
