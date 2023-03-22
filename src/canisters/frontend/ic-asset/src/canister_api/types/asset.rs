use std::collections::HashMap;

use candid::CandidType;
use serde::Deserialize;

/// Information about a content encoding stored for an asset.
#[derive(CandidType, Debug, Deserialize)]
pub struct AssetEncodingDetails {
    /// A content encoding, such as "gzip".
    pub content_encoding: String,

    /// By convention, the sha256 of the entire asset encoding.  This is calculated
    /// by the asset uploader.  It is not generated or validated by the canister.
    pub sha256: Option<Vec<u8>>,
}

/// Information about an asset stored in the canister.
#[derive(CandidType, Debug, Deserialize)]
pub struct AssetDetails {
    /// The key identifies the asset.
    pub key: String,
    /// A list of the encodings stored for the asset.
    pub encodings: Vec<AssetEncodingDetails>,
    /// The MIME type of the asset.
    pub content_type: String,
}

/// TODO add comments
#[derive(Debug, CandidType)]
pub struct SetAssetPropertiesArguments {
    pub key: String,
    pub max_age: Option<Option<u64>>,
    pub headers: Option<Option<Vec<(String, String)>>>,
    pub allow_raw_access: Option<Option<bool>>,
}

/// TODO: comment
#[derive(CandidType, Debug, Deserialize, Default)]
pub struct AssetProperties {
    /// TODO: comment
    pub max_age: Option<u64>,
    /// TODO: comment
    pub headers: Option<HashMap<String, String>>,
    /// TODO: comment
    pub allow_raw_access: Option<bool>,
}

/// TODO: comment
#[derive(CandidType, Debug)]
pub struct GetAssetProperties {
    pub key: String,
}
