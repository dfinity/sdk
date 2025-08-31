use std::collections::{BTreeSet, HashMap};

use candid::{CandidType, Deserialize, Principal};

use crate::{
    asset_certification::types::{certification::CertificateExpression, rc_bytes::RcBytes},
    state_machine::{Asset, AssetEncoding, Configuration, State, Timestamp},
    types::BatchId,
};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableStateV1 {
    pub authorized: Vec<Principal>, // ignored if permissions is Some(_)
    pub permissions: Option<StableStatePermissionsV1>,
    pub stable_assets: HashMap<String, StableAssetV1>,

    pub next_batch_id: Option<BatchId>,
    pub configuration: Option<StableConfigurationV1>,
}

impl StableStateV1 {
    pub fn estimate_size(&self) -> usize {
        let mut size = 0;
        size += 2 + self.authorized.len() * std::mem::size_of::<Principal>();
        size += 1 + self.permissions.as_ref().map_or(0, |p| p.estimate_size());
        size += self.stable_assets.iter().fold(2, |acc, (name, asset)| {
            acc + 2 + name.len() + asset.estimate_size()
        });
        size += 1 + self.next_batch_id.as_ref().map_or(0, |_| 8);
        size += 1 + self.configuration.as_ref().map_or(0, |c| c.estimate_size());
        size
    }
}

impl From<State> for StableStateV1 {
    fn from(state: State) -> Self {
        let permissions = StableStatePermissionsV1 {
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
            next_batch_id: Some(state.next_batch_id),
            configuration: Some(state.configuration.into()),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableStatePermissionsV1 {
    pub commit: BTreeSet<Principal>,
    pub prepare: BTreeSet<Principal>,
    pub manage_permissions: BTreeSet<Principal>,
}

impl StableStatePermissionsV1 {
    fn estimate_size(&self) -> usize {
        8 + self.commit.len() * std::mem::size_of::<Principal>()
            + 8
            + self.prepare.len() * std::mem::size_of::<Principal>()
            + 8
            + self.manage_permissions.len() * std::mem::size_of::<Principal>()
    }
}

/// Same as [super::Configuration] but Candid-serializable
#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct StableConfigurationV1 {
    pub max_batches: Option<u64>,
    pub max_chunks: Option<u64>,
    pub max_bytes: Option<u64>,
}

impl StableConfigurationV1 {
    fn estimate_size(&self) -> usize {
        1 + self
            .max_batches
            .as_ref()
            .map_or(0, |_| std::mem::size_of::<u64>())
            + 1
            + self
                .max_chunks
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<u64>())
            + 1
            + self
                .max_bytes
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<u64>())
    }
}

impl From<Configuration> for StableConfigurationV1 {
    fn from(configuration: Configuration) -> Self {
        Self {
            max_batches: configuration.max_batches,
            max_chunks: configuration.max_chunks,
            max_bytes: configuration.max_bytes,
        }
    }
}

impl From<StableConfigurationV1> for Configuration {
    fn from(stable_configuration: StableConfigurationV1) -> Self {
        Self {
            max_batches: stable_configuration.max_batches,
            max_chunks: stable_configuration.max_chunks,
            max_bytes: stable_configuration.max_bytes,
        }
    }
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

impl StableAssetV1 {
    fn estimate_size(&self) -> usize {
        let mut size = 0;
        size += 1 + self.content_type.len();
        size += self.encodings.iter().fold(1, |acc, (name, encoding)| {
            acc + 1 + name.len() + encoding.estimate_size()
        });
        size += 1 + self
            .max_age
            .as_ref()
            .map_or(0, |_| std::mem::size_of::<u64>());
        size += 1 + self.headers.as_ref().map_or(0, |hm| {
            hm.iter()
                .fold(2, |acc, (k, v)| acc + 1 + k.len() + 2 + v.len())
        });
        size += 1 + self
            .is_aliased
            .as_ref()
            .map_or(0, |_| std::mem::size_of::<bool>());
        size += 1 + self
            .allow_raw_access
            .as_ref()
            .map_or(0, |_| std::mem::size_of::<bool>());
        size
    }
}

impl From<Asset> for StableAssetV1 {
    fn from(asset: Asset) -> Self {
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

impl From<StableAssetV1> for Asset {
    fn from(stable_asset: StableAssetV1) -> Self {
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

impl StableAssetEncodingV1 {
    fn estimate_size(&self) -> usize {
        let mut size = 0;
        size += 8; // modified
        size += self.total_length + self.content_chunks.len() * 4;
        size += 5; // total_length
        size += 1; //  certified
        size += self.sha256.len();
        size += 1 + self
            .certificate_expression
            .as_ref()
            .map_or(0, |ce| 2 + ce.expression.len() + ce.expression_hash.len());
        size += 1 + self.response_hashes.as_ref().map_or(0, |hashes| {
            hashes.iter().fold(2, |acc, (_k, v)| acc + 2 + v.len())
        });
        size
    }
}

impl From<AssetEncoding> for StableAssetEncodingV1 {
    fn from(asset_encoding: AssetEncoding) -> Self {
        Self {
            modified: asset_encoding.modified,
            content_chunks: asset_encoding.content_chunks,
            total_length: asset_encoding.total_length,
            certified: asset_encoding.certified,
            sha256: asset_encoding.sha256,
            certificate_expression: asset_encoding.certificate_expression,
            response_hashes: asset_encoding.response_hashes,
        }
    }
}

impl From<StableAssetEncodingV1> for AssetEncoding {
    fn from(stable_asset_encoding: StableAssetEncodingV1) -> Self {
        Self {
            modified: stable_asset_encoding.modified,
            content_chunks: stable_asset_encoding.content_chunks,
            total_length: stable_asset_encoding.total_length,
            certified: stable_asset_encoding.certified,
            sha256: stable_asset_encoding.sha256,
            certificate_expression: stable_asset_encoding.certificate_expression,
            response_hashes: stable_asset_encoding.response_hashes,
        }
    }
}
