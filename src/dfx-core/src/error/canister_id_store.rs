use crate::error::{
    config::GetTempPathError,
    dfx_config::GetPullCanistersError, fs::FsError, load_dfx_config::LoadDfxConfigError,
    structured_file::StructuredFileError, unified_io::UnifiedIoError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanisterIdStoreError {
    #[error(transparent)]
    UnifiedIoError(#[from] UnifiedIoError),

    #[error(
        "Cannot find canister id. Please issue 'dfx canister create {canister_name}{network}'."
    )]
    CanisterIdNotFound {
        canister_name: String,
        network: String,
    },

    #[error("Encountered error while loading canister id store for network '{network}' - ensuring cohesive network directory failed: {cause}")]
    EnsureCohesiveNetworkDirectoryFailed {
        network: String,
        cause: UnifiedIoError,
    },

    #[error("Failed to remove canister '{canister_name}' from id store: {cause}")]
    RemoveCanisterId {
        canister_name: String,
        cause: UnifiedIoError,
    },

    #[error("Failed to add canister with name '{canister_name}' and id '{canister_id}' to canister id store: {cause}")]
    AddCanisterId {
        canister_name: String,
        canister_id: String,
        cause: UnifiedIoError,
    },

    #[error(transparent)]
    GetPullCanistersFailed(#[from] GetPullCanistersError),

    #[error(transparent)]
    GetTempPath(#[from] GetTempPathError),

    #[error(transparent)]
    LoadDfxConfig(#[from] LoadDfxConfigError),
}

impl From<FsError> for CanisterIdStoreError {
    fn from(e: FsError) -> Self {
        Into::<UnifiedIoError>::into(e).into()
    }
}

impl From<StructuredFileError> for CanisterIdStoreError {
    fn from(e: StructuredFileError) -> Self {
        Into::<UnifiedIoError>::into(e).into()
    }
}
