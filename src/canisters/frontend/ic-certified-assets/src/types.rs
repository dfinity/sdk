//! This module defines types shared by the certified assets state machine and the canister
//! endpoints.
use std::collections::HashMap;

use crate::rc_bytes::RcBytes;
use candid::{CandidType, Deserialize, Nat, Principal};
use serde_bytes::ByteBuf;

pub type BatchId = Nat;
pub type ChunkId = Nat;
pub type Key = String;

// IDL Types

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CreateAssetArguments {
    pub key: Key,
    pub content_type: String,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub enable_aliasing: Option<bool>,
    pub allow_raw_access: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SetAssetContentArguments {
    pub key: Key,
    pub content_encoding: String,
    pub chunk_ids: Vec<ChunkId>,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct UnsetAssetContentArguments {
    pub key: Key,
    pub content_encoding: String,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct DeleteAssetArguments {
    pub key: Key,
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
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CommitBatchArguments {
    pub batch_id: BatchId,
    pub operations: Vec<BatchOperation>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StoreArg {
    pub key: Key,
    pub content_type: String,
    pub content_encoding: String,
    pub content: ByteBuf,
    pub sha256: Option<ByteBuf>,
    pub aliased: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetArg {
    pub key: Key,
    pub accept_encodings: Vec<String>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct GetChunkArg {
    pub key: Key,
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
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct SetAssetPropertiesArguments {
    pub key: Key,
    pub max_age: Option<Option<u64>>,
    pub headers: Option<Option<HashMap<String, String>>>,
    pub allow_raw_access: Option<Option<bool>>,
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
