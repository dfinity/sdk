use thiserror::Error;

#[derive(Error, Debug)]
pub enum StructuredFileOrFilesystemError {
    #[error(transparent)]
    Io(#[from] dfx_core::error::io::IoError),

    #[error(transparent)]
    StructuredFile(#[from] dfx_core::error::structured_file::StructuredFileError),
}

#[derive(Error, Debug)]
pub enum CanisterIdStoreError {
    #[error(transparent)]
    StructuredFileOrFilesystem(#[from] StructuredFileOrFilesystemError),

    #[error(
        "Cannot find canister id. Please issue 'dfx canister create {canister_name}{network}'."
    )]
    CanisterIdNotFound {
        canister_name: String,
        network: String,
    },

    #[error("Encountered error while loading canister id store for network '{network_descriptor_name}' - ensuring cohesive network directory failed: {cause}")]
    EnsureCohesiveNetworkDirectoryFailed {
        network_descriptor_name: String,
        cause: StructuredFileOrFilesystemError,
    },

    #[error("Failed to remove canister '{canister_name}' from id store: {cause}")]
    RemoveCanisterId {
        canister_name: String,
        cause: StructuredFileOrFilesystemError,
    },

    #[error("Failed to add canister with name '{canister_name}' and id '{canister_id}' to canister id store: {cause}")]
    AddCanisterId {
        canister_name: String,
        canister_id: String,
        cause: StructuredFileOrFilesystemError,
    },
}
