use std::collections::{BTreeSet, HashMap};

use candid::Principal;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    asset_certification::types::{certification::CertificateExpression, rc_bytes::RcBytes},
    types::BatchId,
};

/// Same as [super::StableState] but serializable with cbor
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableState {
    pub(super) authorized: Vec<Principal>, // ignored if permissions is Some(_)
    pub(super) permissions: Option<StableStatePermissions>,
    pub(super) stable_assets: HashMap<String, StableAsset>,

    pub(super) next_batch_id: Option<u64>,
    pub(super) configuration: Option<StableConfiguration>,
}

impl From<super::StableState> for StableState {
    fn from(stable_state: super::StableState) -> Self {
        Self {
            authorized: stable_state.authorized,
            permissions: stable_state.permissions.map(Into::into),
            stable_assets: stable_state
                .stable_assets
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            next_batch_id: stable_state.next_batch_id.map(batch_id_to_u64),
            configuration: stable_state.configuration.map(Into::into),
        }
    }
}

impl From<super::State> for StableState {
    fn from(state: super::State) -> Self {
        let permissions = StableStatePermissions {
            commit: state.commit_principals,
            prepare: state.prepare_principals,
            manage_permissions: state.manage_permissions_principals,
        };
        Self {
            authorized: vec![],
            permissions: Some(permissions),
            stable_assets: state
                .assets
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            next_batch_id: Some(batch_id_to_u64(state.next_batch_id)),
            configuration: Some(state.configuration.into()),
        }
    }
}

/// Same as [super::StableStatePermissions] but serializable with cbor
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableStatePermissions {
    pub(super) commit: BTreeSet<Principal>,
    pub(super) prepare: BTreeSet<Principal>,
    pub(super) manage_permissions: BTreeSet<Principal>,
}

impl From<super::StableStatePermissions> for StableStatePermissions {
    fn from(stable_state_permissions: super::StableStatePermissions) -> Self {
        Self {
            commit: stable_state_permissions.commit,
            prepare: stable_state_permissions.prepare,
            manage_permissions: stable_state_permissions.manage_permissions,
        }
    }
}

/// Same as [super::Configuration] but serializable with cbor
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableConfiguration {
    pub max_batches: Option<u64>,
    pub max_chunks: Option<u64>,
    pub max_bytes: Option<u64>,
}

impl From<super::Configuration> for StableConfiguration {
    fn from(configuration: super::Configuration) -> Self {
        Self {
            max_batches: configuration.max_batches,
            max_chunks: configuration.max_chunks,
            max_bytes: configuration.max_bytes,
        }
    }
}

impl From<StableConfiguration> for super::Configuration {
    fn from(stable_configuration: StableConfiguration) -> Self {
        Self {
            max_batches: stable_configuration.max_batches,
            max_chunks: stable_configuration.max_chunks,
            max_bytes: stable_configuration.max_bytes,
        }
    }
}

/// Same as [super::Asset] but serializable with cbor
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableAsset {
    pub content_type: String,
    pub encodings: HashMap<String, StableAssetEncoding>,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub is_aliased: Option<bool>,
    pub allow_raw_access: Option<bool>,
}

impl From<super::Asset> for StableAsset {
    fn from(asset: super::Asset) -> Self {
        Self {
            content_type: asset.content_type,
            encodings: asset
                .encodings
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            max_age: asset.max_age,
            headers: asset.headers,
            is_aliased: asset.is_aliased,
            allow_raw_access: asset.allow_raw_access,
        }
    }
}

impl From<StableAsset> for super::Asset {
    fn from(stable_asset: StableAsset) -> Self {
        Self {
            content_type: stable_asset.content_type,
            encodings: stable_asset
                .encodings
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            max_age: stable_asset.max_age,
            headers: stable_asset.headers,
            is_aliased: stable_asset.is_aliased,
            allow_raw_access: stable_asset.allow_raw_access,
        }
    }
}

/// Same as [super::AssetEncoding] but serializable with cbor
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct StableAssetEncoding {
    pub modified: u64,
    pub content_chunks: Vec<RcBytes>,
    pub total_length: usize,
    pub certified: bool,
    pub sha256: [u8; 32],
    pub certificate_expression: Option<CertificateExpression>,
    pub response_hashes: Option<HashMap<u16, [u8; 32]>>,
}

impl From<super::AssetEncoding> for StableAssetEncoding {
    fn from(asset_encoding: super::AssetEncoding) -> Self {
        Self {
            modified: timestamp_to_u64(asset_encoding.modified),
            content_chunks: asset_encoding.content_chunks,
            total_length: asset_encoding.total_length,
            certified: asset_encoding.certified,
            sha256: asset_encoding.sha256,
            certificate_expression: asset_encoding.certificate_expression,
            response_hashes: asset_encoding.response_hashes,
        }
    }
}

impl From<StableAssetEncoding> for super::AssetEncoding {
    fn from(stable_asset_encoding: StableAssetEncoding) -> Self {
        Self {
            modified: super::Timestamp::from(stable_asset_encoding.modified),
            content_chunks: stable_asset_encoding.content_chunks,
            total_length: stable_asset_encoding.total_length,
            certified: stable_asset_encoding.certified,
            sha256: stable_asset_encoding.sha256,
            certificate_expression: stable_asset_encoding.certificate_expression,
            response_hashes: stable_asset_encoding.response_hashes,
        }
    }
}

fn timestamp_to_u64(timestamp: super::Timestamp) -> u64 {
    timestamp.0.to_u64().expect("timestamp overflow")
}

fn batch_id_to_u64(batch_id: BatchId) -> u64 {
    batch_id.0.to_u64().expect("batch id overflow")
}
