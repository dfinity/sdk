use crate::error::fs::{ReadToStringError, RemoveDirectoryAndContentsError, WriteFileError};
use crate::error::{
    config::GetTempPathError,
    dfx_config::GetPullCanistersError,
    fs::{CreateDirAllError, EnsureParentDirExistsError},
    load_dfx_config::LoadDfxConfigError,
    structured_file::StructuredFileError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanisterIdStoreError {
    #[error(
        "Cannot find canister id. Please issue 'dfx canister create {canister_name}{network}'."
    )]
    CanisterIdNotFound {
        canister_name: String,
        network: String,
    },

    #[error("failed to ensure cohesive network directory")]
    EnsureCohesiveNetworkDirectoryFailed(#[from] EnsureCohesiveNetworkDirectoryError),

    #[error(transparent)]
    RemoveCanisterId(#[from] RemoveCanisterIdError),

    #[error(transparent)]
    GetPullCanistersFailed(#[from] GetPullCanistersError),

    #[error(transparent)]
    GetTempPath(#[from] GetTempPathError),

    #[error(transparent)]
    LoadDfxConfig(#[from] LoadDfxConfigError),

    #[error(transparent)]
    StructuredFileError(#[from] StructuredFileError),
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
pub enum EnsureCohesiveNetworkDirectoryError {
    #[error(transparent)]
    CreateDirAll(#[from] CreateDirAllError),

    #[error(transparent)]
    ReadToString(#[from] ReadToStringError),

    #[error(transparent)]
    RemoveDirAll(#[from] RemoveDirectoryAndContentsError),

    #[error(transparent)]
    Write(#[from] WriteFileError),
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
    EnsureParentDirExists(#[from] EnsureParentDirExistsError),

    #[error(transparent)]
    SaveJsonFile(#[from] StructuredFileError),
}

#[derive(Error, Debug)]
pub enum SaveIdsError {
    #[error(transparent)]
    EnsureParentDirExists(#[from] EnsureParentDirExistsError),

    #[error(transparent)]
    SaveJsonFile(#[from] StructuredFileError),
}
