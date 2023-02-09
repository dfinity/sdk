use candid::{CandidType, Deserialize};
use serde_cbor::ser::IoWrite;
use serde_cbor::Serializer;

use crate::{tree::NestedTree, types::AssetKey};

pub type AssetHashes = NestedTree<NestedTreeKey, Vec<u8>>;

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct IcCertificateExpression {
    pub ic_certificate_expression: String,
    /// Hash of ic_certificate_expression
    pub expression_hash: Vec<u8>,
}

/// AssetKey that has been split into segments.
/// E.g. `["foo", "index.html"]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetPath(pub Vec<AssetKey>);
/// AssetPath that is ready to be inserted into asset_hashes.
/// E.g. `["http_expr", "foo", "index.html", "<$>", "<expr_hash>", "<response_hash>"]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetHashPath(pub Vec<NestedTreeKey>);

impl<T> From<T> for AssetPath
where
    T: Into<Vec<String>>,
{
    fn from(t: T) -> Self {
        Self(t.into())
    }
}

impl AssetPath {
    pub fn from_asset_key(key: &str) -> Self {
        let mut iter = key.split("/").peekable();
        if let Some(first_segment) = iter.peek() {
            if *first_segment == "" {
                iter.next();
            }
        }
        Self(iter.map(|segment| segment.to_string()).collect())
    }

    pub fn reconstruct_asset_key(&self) -> AssetKey {
        format!("/{}", self.0.join("/"))
    }

    pub fn asset_hash_path_v1(&self) -> AssetHashPath {
        AssetHashPath(vec![
            NestedTreeKey::String("http_assets".to_string()),
            NestedTreeKey::String(self.reconstruct_asset_key()),
        ])
    }

    pub fn asset_hash_path_root_v2(&self) -> AssetHashPath {
        let mut hash_path: Vec<NestedTreeKey> = self
            .0
            .iter()
            .map(|segment| NestedTreeKey::String(segment.into()))
            .collect();
        hash_path.push(NestedTreeKey::String("<$>".to_string()));
        hash_path.insert(0, NestedTreeKey::String("http_expr".to_string()));
        AssetHashPath(hash_path)
    }
}

impl AssetHashPath {
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
}

impl AsRef<[u8]> for NestedTreeKey {
    fn as_ref(&self) -> &[u8] {
        match self {
            NestedTreeKey::String(s) => s.as_bytes(),
            NestedTreeKey::Bytes(b) => b.as_slice(),
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
        Self::Bytes(slice.iter().map(|b| b.clone()).collect())
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
