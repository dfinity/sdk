//! This module defines types shared by the certified assets state machine and the canister
//! endpoints.
use std::collections::HashMap;

use candid::{CandidType, Deserialize, Nat, Principal};
use serde_bytes::ByteBuf;

use crate::asset_certification::types::{certification::AssetKey, rc_bytes::RcBytes};

pub type BatchId = Nat;
pub type ChunkId = Nat;

// IDL Types

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigureArguments {
    pub max_batches: Option<Option<u64>>,
    pub max_chunks: Option<Option<u64>>,
    pub max_bytes: Option<Option<u64>>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ConfigurationResponse {
    pub max_batches: Option<u64>,
    pub max_chunks: Option<u64>,
    pub max_bytes: Option<u64>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateAssetArguments {
    pub key: AssetKey,
    pub content_type: String,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub enable_aliasing: Option<bool>,
    pub allow_raw_access: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SetAssetContentArguments {
    pub key: AssetKey,
    pub content_encoding: String,
    pub chunk_ids: Vec<ChunkId>,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UnsetAssetContentArguments {
    pub key: AssetKey,
    pub content_encoding: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct DeleteAssetArguments {
    pub key: AssetKey,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ClearArguments {}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum BatchOperation {
    CreateAsset(CreateAssetArguments),
    SetAssetContent(SetAssetContentArguments),
    UnsetAssetContent(UnsetAssetContentArguments),
    DeleteAsset(DeleteAssetArguments),
    Clear(ClearArguments),
    SetAssetProperties(SetAssetPropertiesArguments),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CommitBatchArguments {
    pub batch_id: BatchId,
    pub operations: Vec<BatchOperation>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CommitProposedBatchArguments {
    pub batch_id: BatchId,
    pub evidence: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct DeleteBatchArguments {
    pub batch_id: BatchId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ComputeEvidenceArguments {
    pub batch_id: BatchId,
    pub max_iterations: Option<u16>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StoreArg {
    pub key: AssetKey,
    pub content_type: String,
    pub content_encoding: String,
    pub content: ByteBuf,
    pub sha256: Option<ByteBuf>,
    pub aliased: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetArg {
    pub key: AssetKey,
    pub accept_encodings: Vec<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetChunkArg {
    pub key: AssetKey,
    pub content_encoding: String,
    pub index: Nat,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetChunkResponse {
    pub content: RcBytes,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateBatchResponse {
    pub batch_id: BatchId,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateChunkArg {
    pub batch_id: BatchId,
    pub content: ByteBuf,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateChunkResponse {
    pub chunk_id: ChunkId,
}

#[derive(Clone, Debug, CandidType, Deserialize, PartialEq, Eq)]
pub struct AssetProperties {
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub allow_raw_access: Option<bool>,
    pub is_aliased: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SetAssetPropertiesArguments {
    pub key: AssetKey,
    pub max_age: Option<Option<u64>>,
    pub headers: Option<Option<HashMap<String, String>>>,
    pub allow_raw_access: Option<Option<bool>>,
    pub is_aliased: Option<Option<bool>>,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub enum Permission {
    Commit,
    ManagePermissions,
    Prepare,
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Permission::Commit => f.write_str("Commit"),
            Permission::Prepare => f.write_str("Prepare"),
            Permission::ManagePermissions => f.write_str("ManagePermissions"),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GrantPermissionArguments {
    pub to_principal: Principal,
    pub permission: Permission,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RevokePermissionArguments {
    pub of_principal: Principal,
    pub permission: Permission,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ListPermittedArguments {
    pub permission: Permission,
}
