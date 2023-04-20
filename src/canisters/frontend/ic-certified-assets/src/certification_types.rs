use candid::{CandidType, Deserialize};
use serde_cbor::ser::IoWrite;
use serde_cbor::Serializer;

use crate::{tree::NestedTree, types::AssetKey};

pub type AssetHashes = NestedTree<NestedTreeKey, Vec<u8>>;

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct CertificateExpression {
    pub expression: String,
    /// Hash of expression
    pub hash: [u8; 32],
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct RequestHash(Option<[u8; 32]>);

impl Default for RequestHash {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct ResponseHash(pub [u8; 32]);

/// AssetKey that has been split into segments.
/// E.g. `["foo", "index.html"]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetPath(pub Vec<AssetKey>);

/// AssetPath that is ready to be inserted into asset_hashes.
/// E.g. `["http_expr", "foo", "index.html", "<$>", "<expr_hash>", "<request hash>", "<response_hash>"]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashTreePath(pub Vec<NestedTreeKey>);

impl<T> From<T> for AssetPath
where
    T: AsRef<str>,
{
    fn from(key: T) -> Self {
        let mut iter = key.as_ref().split('/').peekable();
        if let Some(first_segment) = iter.peek() {
            // "/path/to/asset".split("/") produces an empty node before "path", therefore we need to skip it
            if first_segment.is_empty() {
                iter.next();
            }
        }
        Self(iter.map(|segment| segment.to_string()).collect())
    }
}

impl AssetPath {
    pub fn reconstruct_asset_key(&self) -> AssetKey {
        // this reconstructs "" as "/", but this is not a problem because no http client actually requests ""
        format!("/{}", self.0.join("/"))
    }

    pub fn asset_hash_path_v1(&self) -> HashTreePath {
        HashTreePath(vec![
            "http_assets".into(),
            self.reconstruct_asset_key().into(),
        ])
    }

    pub fn asset_hash_path_root_v2(&self) -> HashTreePath {
        let mut hash_path: Vec<NestedTreeKey> = self
            .0
            .iter()
            .map(|segment| segment.as_str().into())
            .collect();
        hash_path.push("<$>".into());
        hash_path.insert(0, "http_expr".into());
        HashTreePath(hash_path)
    }

    pub fn hash_tree_path(
        &self,
        certificate_expression: &CertificateExpression,
        RequestHash(maybe_request_hash): &RequestHash,
        ResponseHash(response_hash): &ResponseHash,
    ) -> HashTreePath {
        let mut hash_path: Vec<NestedTreeKey> = Vec::new();
        hash_path.push("http_expr".into());
        hash_path = self.0.iter().fold(hash_path, |mut path, s| {
            path.push(s.as_str().into());
            path
        });
        hash_path.push("<$>".into()); // asset path terminator
        hash_path.push(certificate_expression.hash.into());
        hash_path.push(if let Some(request_hash) = maybe_request_hash {
            request_hash.clone().into()
        } else {
            "".into() // no request certification - use empty node
        });
        hash_path.push(NestedTreeKey::Hash(response_hash.clone()));
        HashTreePath(hash_path)
    }
}

impl From<Vec<NestedTreeKey>> for HashTreePath {
    fn from(vec: Vec<NestedTreeKey>) -> Self {
        Self(vec)
    }
}

impl HashTreePath {
    pub fn as_vec(&self) -> &Vec<NestedTreeKey> {
        &self.0
    }

    pub fn expr_path(&self) -> String {
        let strings = self
            .0
            .iter()
            .map(|key| match key {
                NestedTreeKey::String(k) => k.clone(),
                NestedTreeKey::Bytes(b) => hex::encode(b),
                NestedTreeKey::Hash(h) => hex::encode(h),
            })
            .collect::<Vec<String>>();
        let cbor = serialize_cbor_self_describing(&strings);
        base64::encode(cbor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NestedTreeKey {
    String(String),
    Bytes(Vec<u8>),
    Hash([u8; 32]),
}

impl AsRef<[u8]> for NestedTreeKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            NestedTreeKey::String(s) => s.as_bytes(),
            NestedTreeKey::Bytes(b) => b.as_slice(),
            NestedTreeKey::Hash(h) => h,
        }
    }
}

impl From<&str> for NestedTreeKey {
    fn from(s: &str) -> Self {
        Self::String(s.into())
    }
}

impl From<&[u8]> for NestedTreeKey {
    fn from(slice: &[u8]) -> Self {
        Self::Bytes(slice.to_vec())
    }
}

impl From<[u8; 32]> for NestedTreeKey {
    fn from(hash: [u8; 32]) -> Self {
        Self::Hash(hash)
    }
}

impl From<String> for NestedTreeKey {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

fn serialize_cbor_self_describing<T>(value: &T) -> Vec<u8>
where
    T: serde::Serialize,
{
    let mut vec = Vec::new();
    let mut binding = IoWrite::new(&mut vec);
    let mut s = Serializer::new(&mut binding);
    s.self_describe()
        .expect("Cannot produce self-describing cbor.");
    value
        .serialize(&mut s)
        .expect("Failed to serialize self-describing CBOR.");
    vec
}
