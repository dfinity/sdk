use std::collections::{BTreeSet, HashMap};

use candid::Principal;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

use super::v1::{
    StableAssetEncodingV1, StableAssetV1, StableConfigurationV1, StableStatePermissionsV1,
    StableStateV1,
};
use crate::{
    asset_certification::types::{certification::CertificateExpression, rc_bytes::RcBytes},
    state_machine::Timestamp,
    types::BatchId,
};

/// Same as [StableStateV1] but serde-serializable
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableStateV2 {
    pub(super) authorized: Vec<Principal>, // ignored if permissions is Some(_)
    pub(super) permissions: Option<StableStatePermissionsV2>,
    pub(super) stable_assets: HashMap<String, StableAssetV2>,

    pub(super) next_batch_id: Option<u64>,
    pub(super) configuration: Option<StableConfigurationV2>,
}

impl From<StableStateV1> for StableStateV2 {
    fn from(stable_state: StableStateV1) -> Self {
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

impl From<super::State> for StableStateV2 {
    fn from(state: super::State) -> Self {
        let permissions = StableStatePermissionsV2 {
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

/// Same as [StableStatePermissionsV1] but serde-serializable
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableStatePermissionsV2 {
    pub(super) commit: BTreeSet<Principal>,
    pub(super) prepare: BTreeSet<Principal>,
    pub(super) manage_permissions: BTreeSet<Principal>,
}

impl From<StableStatePermissionsV1> for StableStatePermissionsV2 {
    fn from(stable_state_permissions: StableStatePermissionsV1) -> Self {
        Self {
            commit: stable_state_permissions.commit,
            prepare: stable_state_permissions.prepare,
            manage_permissions: stable_state_permissions.manage_permissions,
        }
    }
}

/// Same as [super::Configuration] but serde-serializable
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableConfigurationV2 {
    pub max_batches: Option<u64>,
    pub max_chunks: Option<u64>,
    pub max_bytes: Option<u64>,
}

impl From<super::Configuration> for StableConfigurationV2 {
    fn from(configuration: super::Configuration) -> Self {
        Self {
            max_batches: configuration.max_batches,
            max_chunks: configuration.max_chunks,
            max_bytes: configuration.max_bytes,
        }
    }
}

impl From<StableConfigurationV1> for StableConfigurationV2 {
    fn from(configuration: StableConfigurationV1) -> Self {
        Self {
            max_batches: configuration.max_batches,
            max_chunks: configuration.max_chunks,
            max_bytes: configuration.max_bytes,
        }
    }
}

impl From<StableConfigurationV2> for super::Configuration {
    fn from(stable_configuration: StableConfigurationV2) -> Self {
        Self {
            max_batches: stable_configuration.max_batches,
            max_chunks: stable_configuration.max_chunks,
            max_bytes: stable_configuration.max_bytes,
        }
    }
}

/// Same as [super::Asset] but serde-serializable
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StableAssetV2 {
    pub content_type: String,
    pub encodings: HashMap<String, StableAssetEncodingV2>,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub is_aliased: Option<bool>,
    pub allow_raw_access: Option<bool>,
}

impl From<super::Asset> for StableAssetV2 {
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

impl From<StableAssetV1> for StableAssetV2 {
    fn from(asset: StableAssetV1) -> Self {
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

impl From<StableAssetV2> for super::Asset {
    fn from(stable_asset: StableAssetV2) -> Self {
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

/// Same as [super::AssetEncoding] but serde-serializable
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct StableAssetEncodingV2 {
    pub modified: u64,
    pub content_chunks: Vec<RcBytes>,
    pub total_length: usize,
    pub certified: bool,
    pub sha256: [u8; 32],
    pub certificate_expression: Option<CertificateExpression>,
    pub response_hashes: Option<HashMap<u16, [u8; 32]>>,
}

impl From<super::AssetEncoding> for StableAssetEncodingV2 {
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

impl From<StableAssetEncodingV1> for StableAssetEncodingV2 {
    fn from(asset_encoding: StableAssetEncodingV1) -> Self {
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

impl From<StableAssetEncodingV2> for super::AssetEncoding {
    fn from(stable_asset_encoding: StableAssetEncodingV2) -> Self {
        Self {
            modified: Timestamp::from(stable_asset_encoding.modified),
            content_chunks: stable_asset_encoding.content_chunks,
            total_length: stable_asset_encoding.total_length,
            certified: stable_asset_encoding.certified,
            sha256: stable_asset_encoding.sha256,
            certificate_expression: stable_asset_encoding.certificate_expression,
            response_hashes: stable_asset_encoding.response_hashes,
        }
    }
}

fn timestamp_to_u64(timestamp: Timestamp) -> u64 {
    timestamp.0.to_u64().expect("timestamp overflow")
}

fn batch_id_to_u64(batch_id: BatchId) -> u64 {
    batch_id.0.to_u64().expect("batch id overflow")
}
