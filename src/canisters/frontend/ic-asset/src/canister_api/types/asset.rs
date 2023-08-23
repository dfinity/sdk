use candid::CandidType;
use serde::Deserialize;
use std::collections::HashMap;

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

/// Information about the properties stored for an asset.
#[derive(CandidType, Debug, Deserialize, Default)]
pub struct AssetProperties {
    /// Asset's cache max_age property
    pub max_age: Option<u64>,
    /// Asset's HTTP response headers
    pub headers: Option<HashMap<String, String>>,
    /// Asset's toggle for whether to serve the asset over .raw domain
    pub allow_raw_access: Option<bool>,
    /// Asset's toggle for whether to serve the .html asset both as /route and /route.html
    pub is_aliased: Option<bool>,
}

/// Sets the asset with the given properties.
#[derive(Debug, Clone, CandidType, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetAssetPropertiesArguments {
    pub key: String,
    pub max_age: Option<Option<u64>>,
    pub headers: Option<Option<Vec<(String, String)>>>,
    pub allow_raw_access: Option<Option<bool>>,
    pub is_aliased: Option<Option<bool>>,
}

/// The arguments to the `get_asset_properties` method.
#[derive(CandidType, Debug)]
pub struct GetAssetPropertiesArgument(pub String);
