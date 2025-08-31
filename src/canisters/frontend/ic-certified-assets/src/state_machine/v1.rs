use std::collections::{BTreeSet, HashMap};

use candid::{CandidType, Deserialize, Nat, Principal};

use crate::{
    asset_certification::types::{certification::CertificateExpression, rc_bytes::RcBytes},
    state_machine::Timestamp,
};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableStateV1 {
    pub authorized: Vec<Principal>,
    pub permissions: Option<StableStatePermissionsV1>,
    pub stable_assets: HashMap<String, StableAssetV1>,
    pub next_batch_id: Option<Nat>,
    pub configuration: Option<StableConfigurationV1>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableStatePermissionsV1 {
    pub commit: BTreeSet<Principal>,
    pub prepare: BTreeSet<Principal>,
    pub manage_permissions: BTreeSet<Principal>,
}

/// Same as [super::Configuration] but Candid-serializable
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct StableConfigurationV1 {
    pub max_batches: Option<u64>,
    pub max_chunks: Option<u64>,
    pub max_bytes: Option<u64>,
}

/// Same as [super::Asset] but Candid-serializable
#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct StableAssetV1 {
    pub content_type: String,
    pub encodings: HashMap<String, StableAssetEncodingV1>,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub is_aliased: Option<bool>,
    pub allow_raw_access: Option<bool>,
}

/// Same as [super::AssetEncoding] but Candid-serializable
#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct StableAssetEncodingV1 {
    pub modified: Timestamp,
    pub content_chunks: Vec<RcBytes>,
    pub total_length: usize,
    pub certified: bool,
    pub sha256: [u8; 32],
    pub certificate_expression: Option<CertificateExpression>,
    pub response_hashes: Option<HashMap<u16, [u8; 32]>>,
}
