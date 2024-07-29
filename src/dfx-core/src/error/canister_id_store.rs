use crate::error::{
    config::GetTempPathError, dfx_config::GetPullCanistersError, fs::FsError,
    load_dfx_config::LoadDfxConfigError, structured_file::StructuredFileError,
    unified_io::UnifiedIoError,
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

    #[error("Encountered error while loading canister id store for network '{network}' - ensuring cohesive network directory failed")]
    EnsureCohesiveNetworkDirectoryFailed {
        network: String,
        source: UnifiedIoError,
    },

    #[error(transparent)]
    RemoveCanisterId(#[from] RemoveCanisterIdError),

    #[error("Failed to add canister with name '{canister_name}' and id '{canister_id}' to canister id store")]
    AddCanisterId {
        canister_name: String,
        canister_id: String,
        source: AddCanisterIdError,
    },

    #[error(transparent)]
    GetPullCanistersFailed(#[from] GetPullCanistersError),

    #[error(transparent)]
    GetTempPath(#[from] GetTempPathError),

    #[error(transparent)]
    LoadDfxConfig(#[from] LoadDfxConfigError),
}

#[derive(Error, Debug)]
pub enum AddCanisterIdError {
    #[error("failed to add canister with name '{canister_name}' and id '{canister_id}' to canister id store")]
    SaveIds {
        canister_name: String,
        canister_id: String,
        source: SaveIdsError,
    },

    #[error(transparent)]
    SaveTimestamps(#[from] SaveTimestampsError),
}

#[derive(Error, Debug)]
pub enum RemoveCanisterIdError {
    #[error("failed to remove canister '{canister_name}' from id store")]
    SaveIds {
        canister_name: String,
        source: SaveIdsError,
    },

    #[error(transparent)]
    SaveTimestamps(#[from] SaveTimestampsError),
}

#[derive(Error, Debug)]
pub enum SaveTimestampsError {
    #[error(transparent)]
    EnsureParentDirExists(FsError),

    #[error(transparent)]
    SaveJsonFile(StructuredFileError),
}

#[derive(Error, Debug)]
pub enum SaveIdsError {
    #[error(transparent)]
    EnsureParentDirExists(FsError),

    #[error(transparent)]
    SaveJsonFile(StructuredFileError),
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
