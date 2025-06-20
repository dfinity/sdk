use candid::Principal;
use ic_certification::Hash;
use serde_bytes::ByteBuf;
use std::collections::BTreeSet;

use crate::{
    asset_certification::types::{
        certification::AssetKey,
        http::{
            CallbackFunc, HttpRequest, HttpResponse, StreamingCallbackHttpResponse,
            StreamingCallbackToken,
        },
        rc_bytes::RcBytes,
    },
    state_machine::{AssetDetails, CertifiedTree, EncodedAsset},
    types::{
        AssetProperties, BatchId, ChunkId, CommitBatchArguments, CommitProposedBatchArguments,
        ComputeEvidenceArguments, ConfigurationResponse, ConfigureArguments, CreateAssetArguments,
        CreateChunkArg, CreateChunksArg, DeleteAssetArguments, DeleteBatchArguments, GetArg,
        GetChunkArg, Permission, SetAssetContentArguments, SetAssetPropertiesArguments,
        SetPermissions, StoreArg, UnsetAssetContentArguments,
    },
    StableState,
};

pub trait AssetCanisterStateTrait {
    fn set_permissions(&mut self, perm: SetPermissions);

    fn grant_permission(&mut self, principal: Principal, permission: &Permission);

    fn revoke_permission(&mut self, principal: Principal, permission: &Permission);

    fn list_permitted(&self, permission: &Permission) -> &BTreeSet<Principal>;

    fn take_ownership(&mut self, controller: Principal);

    fn root_hash(&self) -> Hash;

    fn create_asset(&mut self, arg: CreateAssetArguments) -> Result<(), String>;

    fn set_asset_content(&mut self, arg: SetAssetContentArguments, now: u64) -> Result<(), String>;

    fn unset_asset_content(&mut self, arg: UnsetAssetContentArguments) -> Result<(), String>;

    fn delete_asset(&mut self, arg: DeleteAssetArguments);

    fn clear(&mut self);

    fn has_permission(&self, principal: &Principal, permission: &Permission) -> bool;

    fn can(&self, principal: &Principal, permission: &Permission) -> bool;

    fn retrieve(&self, key: &AssetKey) -> Result<RcBytes, String>;

    fn store(&mut self, arg: StoreArg, time: u64) -> Result<(), String>;

    fn create_batch(&mut self, now: u64) -> Result<BatchId, String>;

    fn create_chunk(&mut self, arg: CreateChunkArg, now: u64) -> Result<ChunkId, String>;

    fn create_chunks(&mut self, arg: CreateChunksArg, now: u64) -> Result<Vec<ChunkId>, String>;

    fn commit_batch(&mut self, arg: CommitBatchArguments, now: u64) -> Result<(), String>;

    fn propose_commit_batch(&mut self, arg: CommitBatchArguments) -> Result<(), String>;

    fn commit_proposed_batch(
        &mut self,
        arg: CommitProposedBatchArguments,
        now: u64,
    ) -> Result<(), String>;

    fn validate_commit_proposed_batch(
        &self,
        arg: CommitProposedBatchArguments,
    ) -> Result<String, String>;

    fn compute_evidence(
        &mut self,
        arg: ComputeEvidenceArguments,
    ) -> Result<Option<ByteBuf>, String>;

    fn delete_batch(&mut self, arg: DeleteBatchArguments) -> Result<(), String>;

    fn list_assets(&self) -> Vec<AssetDetails>;

    fn certified_tree(&self, certificate: &[u8]) -> CertifiedTree;

    fn get(&self, arg: GetArg) -> Result<EncodedAsset, String>;

    fn get_chunk(&self, arg: GetChunkArg) -> Result<RcBytes, String>;
    fn http_request(
        &self,
        req: HttpRequest,
        certificate: &[u8],
        callback: CallbackFunc,
    ) -> HttpResponse;

    fn http_request_streaming_callback(
        &self,
        arg: StreamingCallbackToken,
    ) -> Result<StreamingCallbackHttpResponse, String>;

    fn get_asset_properties(&self, key: AssetKey) -> Result<AssetProperties, String>;

    fn set_asset_properties(&mut self, arg: SetAssetPropertiesArguments) -> Result<(), String>;

    fn get_configuration(&self) -> ConfigurationResponse;

    fn configure(&mut self, args: ConfigureArguments);

    fn to_stable_state(&self) -> StableState;
}
