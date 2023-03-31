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
