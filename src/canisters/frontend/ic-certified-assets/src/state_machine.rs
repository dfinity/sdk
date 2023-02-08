//! This module contains a pure implementation of the certified assets state machine.

// NB. This module should not depend on ic_cdk, it contains only pure state transition functions.
// All the environment (time, certificates, etc.) is passed to the state transition functions
// as formal arguments.  This approach makes it very easy to test the state machine.

use crate::{
    http::{
        build_ic_certificate_expression_from_headers_and_encoding, HeaderField, HttpRequest,
        HttpResponse, StreamingCallbackHttpResponse, StreamingCallbackToken,
    },
    rc_bytes::RcBytes,
    tree::{merge_hash_trees, NestedTree},
    types::*,
    url_decode::url_decode,
};
use candid::{CandidType, Deserialize, Func, Int, Nat, Principal};
use ic_certified_map::{AsHashTree, Hash, HashTree};
use ic_response_verification::hash::{representation_independent_hash, Value};
use num_traits::ToPrimitive;
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_cbor::{ser::IoWrite, Serializer};
use sha2::Digest;
use std::collections::HashMap;
use std::convert::TryInto;

/// The amount of time a batch is kept alive. Modifying the batch
/// delays the expiry further.
pub const BATCH_EXPIRY_NANOS: u64 = 300_000_000_000;

/// The order in which we pick encodings for certification.
const ENCODING_CERTIFICATION_ORDER: &[&str] = &["identity", "gzip", "compress", "deflate", "br"];

/// The file to serve if the requested file wasn't found.
const INDEX_FILE: &str = "/index.html";

/// Default aliasing behavior.
const DEFAULT_ALIAS_ENABLED: bool = true;

type AssetHashes = NestedTree<NestedTreeKey, Vec<u8>>;
type Timestamp = Int;

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct IcCertificateExpression {
    pub ic_certificate_expression: String,
    /// Hash of ic_certificate_expression
    pub expression_hash: Vec<u8>,
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct AssetEncoding {
    pub modified: Timestamp,
    pub content_chunks: Vec<RcBytes>,
    pub total_length: usize,
    pub certified: bool,
    pub sha256: [u8; 32],
    pub ic_ce: Option<IcCertificateExpression>,
    pub response_hash: Option<[u8; 32]>,
}
impl AssetEncoding {
    fn asset_hash_path_v2(&self, AssetPath(path): AssetPath) -> Option<AssetHashPath> {
        if let Some(ce) = self.ic_ce.as_ref() {
            if let Some(response_hash) = self.response_hash.as_ref() {
                let mut path: Vec<NestedTreeKey> = path
                    .into_iter()
                    .map(|segment| NestedTreeKey::String(segment))
                    .collect();
                path.insert(0, NestedTreeKey::String("http_expr".to_string()));
                path.push(NestedTreeKey::String("<$>".to_string()));
                path.push(NestedTreeKey::Bytes(ce.expression_hash.clone()));
                path.push(response_hash.as_slice().into());
                Some(AssetHashPath(path))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct Asset {
    pub content_type: String,
    pub encodings: HashMap<String, AssetEncoding>,
    pub max_age: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub is_aliased: Option<bool>,
    pub allow_raw_access: Option<bool>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct EncodedAsset {
    pub content: RcBytes,
    pub content_type: String,
    pub content_encoding: String,
    pub total_length: Nat,
    pub sha256: Option<ByteBuf>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AssetDetails {
    pub key: String,
    pub content_type: String,
    pub encodings: Vec<AssetEncodingDetails>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct AssetEncodingDetails {
    pub content_encoding: String,
    pub sha256: Option<ByteBuf>,
    pub length: Nat,
    pub modified: Timestamp,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CertifiedTree {
    pub certificate: Vec<u8>,
    pub tree: Vec<u8>,
}

pub struct Chunk {
    pub batch_id: BatchId,
    pub content: RcBytes,
}

pub struct Batch {
    pub expires_at: Timestamp,
}

#[derive(Default)]
pub struct State {
    assets: HashMap<AssetKey, Asset>,

    chunks: HashMap<ChunkId, Chunk>,
    next_chunk_id: ChunkId,

    batches: HashMap<BatchId, Batch>,
    next_batch_id: BatchId,

    authorized: Vec<Principal>,

    asset_hashes: AssetHashes,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableState {
    authorized: Vec<Principal>,
    stable_assets: HashMap<String, Asset>,
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

impl Asset {
    fn allow_raw_access(&self) -> bool {
        self.allow_raw_access.unwrap_or(false)
    }

    fn update_ic_certificate_expressions(&mut self) {
        // gather all headers
        let mut headers = vec![];

        if self.max_age.is_some() {
            headers.push("cache-control");
        }
        if let Some(custom_headers) = &self.headers {
            for (k, _) in custom_headers.iter() {
                headers.push(k);
            }
        }

        // update
        for (enc_name, encoding) in self.encodings.iter_mut() {
            encoding.ic_ce = Some(build_ic_certificate_expression_from_headers_and_encoding(
                &headers, enc_name,
            ));
        }
    }

    pub fn get_headers_for_asset(&self, encoding_name: &str) -> HashMap<String, String> {
        build_headers(
            self.headers.as_ref().map(|h| h.iter()),
            &self.max_age,
            &self.content_type,
            encoding_name.to_owned(),
            self.encodings
                .get(encoding_name)
                .and_then(|e| e.ic_ce.as_ref().map(|ce| &ce.ic_certificate_expression)),
        )
    }
}

impl State {
    fn get_asset(&self, key: &AssetKey) -> Result<&Asset, String> {
        self.assets
            .get(key)
            .or_else(|| {
                let aliased = aliases_of_key(key)
                    .into_iter()
                    .find_map(|alias_key| self.assets.get(&alias_key));
                if let Some(asset) = aliased {
                    if asset.is_aliased.unwrap_or(DEFAULT_ALIAS_ENABLED) {
                        aliased
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or_else(|| "asset not found".to_string())
    }

    pub fn authorize_unconditionally(&mut self, principal: Principal) {
        if !self.is_authorized(&principal) {
            self.authorized.push(principal);
        }
    }

    pub fn deauthorize_unconditionally(&mut self, principal: Principal) {
        if let Some(pos) = self.authorized.iter().position(|x| *x == principal) {
            self.authorized.remove(pos);
        }
    }

    pub fn list_authorized(&self) -> &Vec<Principal> {
        &self.authorized
    }

    pub fn root_hash(&self) -> Hash {
        self.asset_hashes.root_hash()
    }

    pub fn create_asset(&mut self, arg: CreateAssetArguments) -> Result<(), String> {
        if let Some(asset) = self.assets.get(&arg.key) {
            if asset.content_type != arg.content_type {
                return Err("create_asset: content type mismatch".to_string());
            }
        } else {
            self.assets.insert(
                arg.key,
                Asset {
                    content_type: arg.content_type,
                    encodings: HashMap::new(),
                    max_age: arg.max_age,
                    headers: arg.headers,
                    is_aliased: arg.enable_aliasing,
                    allow_raw_access: arg.allow_raw_access,
                },
            );
        }
        Ok(())
    }

    pub fn set_asset_content(
        &mut self,
        arg: SetAssetContentArguments,
        now: u64,
    ) -> Result<(), String> {
        if arg.chunk_ids.is_empty() {
            return Err("encoding must have at least one chunk".to_string());
        }

        let dependent_keys = self.dependent_keys_v1(&arg.key);
        let asset = self
            .assets
            .get_mut(&arg.key)
            .ok_or_else(|| "asset not found".to_string())?;

        let now = Int::from(now);

        let mut content_chunks = vec![];
        for chunk_id in arg.chunk_ids.iter() {
            let chunk = self.chunks.remove(chunk_id).expect("chunk not found");
            content_chunks.push(chunk.content);
        }

        let sha256: [u8; 32] = match arg.sha256 {
            Some(bytes) => bytes
                .into_vec()
                .try_into()
                .map_err(|_| "invalid SHA-256".to_string())?,
            None => {
                let mut hasher = sha2::Sha256::new();
                for chunk in content_chunks.iter() {
                    hasher.update(chunk);
                }
                hasher.finalize().into()
            }
        };

        let total_length: usize = content_chunks.iter().map(|c| c.len()).sum();
        let enc = AssetEncoding {
            modified: now,
            content_chunks,
            certified: false,
            total_length,
            sha256,
            ic_ce: None,         // set by on_asset_change
            response_hash: None, // set by on_asset_change
        };
        asset.encodings.insert(arg.content_encoding, enc);

        on_asset_change(&mut self.asset_hashes, &arg.key, asset, dependent_keys);

        Ok(())
    }

    pub fn unset_asset_content(&mut self, arg: UnsetAssetContentArguments) -> Result<(), String> {
        let dependent_keys = self.dependent_keys_v1(&arg.key);
        let asset = self
            .assets
            .get_mut(&arg.key)
            .ok_or_else(|| "asset not found".to_string())?;

        if asset.encodings.remove(&arg.content_encoding).is_some() {
            on_asset_change(&mut self.asset_hashes, &arg.key, asset, dependent_keys);
        }

        Ok(())
    }

    pub fn delete_asset(&mut self, arg: DeleteAssetArguments) {
        if let Some(_) = self.assets.get(&arg.key) {
            for dependent in self.dependent_keys_v1(&arg.key) {
                let path = AssetPath::from_asset_key(&dependent);
                self.asset_hashes.delete(path.asset_hash_path_v1().as_vec());
                self.asset_hashes
                    .delete(path.asset_hash_path_root_v2().as_vec());
            }
            self.assets.remove(&arg.key);
        }
    }

    pub fn clear(&mut self) {
        self.assets.clear();
        self.batches.clear();
        self.chunks.clear();
        self.next_batch_id = Nat::from(1);
        self.next_chunk_id = Nat::from(1);
    }

    pub fn is_authorized(&self, principal: &Principal) -> bool {
        self.authorized.contains(principal)
    }

    pub fn retrieve(&self, key: &AssetKey) -> Result<RcBytes, String> {
        let asset = self.get_asset(key)?;

        let id_enc = asset
            .encodings
            .get("identity")
            .ok_or_else(|| "no identity encoding".to_string())?;

        if id_enc.content_chunks.len() > 1 {
            return Err("Asset too large. Use get() and get_chunk() instead.".to_string());
        }

        Ok(id_enc.content_chunks[0].clone())
    }

    pub fn store(&mut self, arg: StoreArg, time: u64) -> Result<(), String> {
        let dependent_keys = self.dependent_keys_v1(&arg.key);
        let asset = self.assets.entry(arg.key.clone()).or_default();
        asset.content_type = arg.content_type;
        asset.is_aliased = arg.aliased;

        let hash = sha2::Sha256::digest(&arg.content).into();
        if let Some(provided_hash) = arg.sha256 {
            if hash != provided_hash.as_ref() {
                return Err("sha256 mismatch".to_string());
            }
        }

        let encoding = asset.encodings.entry(arg.content_encoding).or_default();
        encoding.total_length = arg.content.len();
        encoding.content_chunks = vec![RcBytes::from(arg.content)];
        encoding.modified = Int::from(time);
        encoding.sha256 = hash;

        on_asset_change(&mut self.asset_hashes, &arg.key, asset, dependent_keys);
        Ok(())
    }

    pub fn create_batch(&mut self, now: u64) -> BatchId {
        let batch_id = self.next_batch_id.clone();
        self.next_batch_id += 1;

        self.batches.insert(
            batch_id.clone(),
            Batch {
                expires_at: Int::from(now + BATCH_EXPIRY_NANOS),
            },
        );
        self.chunks.retain(|_, c| {
            self.batches
                .get(&c.batch_id)
                .map(|b| b.expires_at > now)
                .unwrap_or(false)
        });
        self.batches.retain(|_, b| b.expires_at > now);

        batch_id
    }

    pub fn create_chunk(&mut self, arg: CreateChunkArg, now: u64) -> Result<ChunkId, String> {
        let mut batch = self
            .batches
            .get_mut(&arg.batch_id)
            .ok_or_else(|| "batch not found".to_string())?;

        batch.expires_at = Int::from(now + BATCH_EXPIRY_NANOS);

        let chunk_id = self.next_chunk_id.clone();
        self.next_chunk_id += 1;

        self.chunks.insert(
            chunk_id.clone(),
            Chunk {
                batch_id: arg.batch_id,
                content: RcBytes::from(arg.content),
            },
        );

        Ok(chunk_id)
    }

    pub fn commit_batch(&mut self, arg: CommitBatchArguments, now: u64) -> Result<(), String> {
        let batch_id = arg.batch_id;
        for op in arg.operations {
            match op {
                BatchOperation::CreateAsset(arg) => self.create_asset(arg)?,
                BatchOperation::SetAssetContent(arg) => self.set_asset_content(arg, now)?,
                BatchOperation::UnsetAssetContent(arg) => self.unset_asset_content(arg)?,
                BatchOperation::DeleteAsset(arg) => self.delete_asset(arg),
                BatchOperation::Clear(_) => self.clear(),
            }
        }
        self.batches.remove(&batch_id);
        Ok(())
    }

    pub fn list_assets(&self) -> Vec<AssetDetails> {
        self.assets
            .iter()
            .map(|(key, asset)| {
                let mut encodings: Vec<_> = asset
                    .encodings
                    .iter()
                    .map(|(enc_name, enc)| AssetEncodingDetails {
                        content_encoding: enc_name.clone(),
                        sha256: Some(ByteBuf::from(enc.sha256)),
                        length: Nat::from(enc.total_length),
                        modified: enc.modified.clone(),
                    })
                    .collect();
                encodings.sort_by(|l, r| l.content_encoding.cmp(&r.content_encoding));

                AssetDetails {
                    key: key.clone(),
                    content_type: asset.content_type.clone(),
                    encodings,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn certified_tree(&self, certificate: &[u8]) -> CertifiedTree {
        let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
        serializer.self_describe().unwrap();
        self.asset_hashes
            .as_hash_tree()
            .serialize(&mut serializer)
            .unwrap();

        CertifiedTree {
            certificate: certificate.to_vec(),
            tree: serializer.into_inner(),
        }
    }

    pub fn get(&self, arg: GetArg) -> Result<EncodedAsset, String> {
        let asset = self.get_asset(&arg.key)?;

        for enc in arg.accept_encodings.iter() {
            if let Some(asset_enc) = asset.encodings.get(enc) {
                return Ok(EncodedAsset {
                    content: asset_enc.content_chunks[0].clone(),
                    content_type: asset.content_type.clone(),
                    content_encoding: enc.clone(),
                    total_length: Nat::from(asset_enc.total_length as u64),
                    sha256: Some(ByteBuf::from(asset_enc.sha256)),
                });
            }
        }
        Err("no such encoding".to_string())
    }

    pub fn get_chunk(&self, arg: GetChunkArg) -> Result<RcBytes, String> {
        let asset = self.get_asset(&arg.key)?;

        let enc = asset
            .encodings
            .get(&arg.content_encoding)
            .ok_or_else(|| "no such encoding".to_string())?;

        if let Some(expected_hash) = arg.sha256 {
            if expected_hash != enc.sha256 {
                return Err("sha256 mismatch".to_string());
            }
        }
        if arg.index >= enc.content_chunks.len() {
            return Err("chunk index out of bounds".to_string());
        }
        let index: usize = arg.index.0.to_usize().unwrap();

        Ok(enc.content_chunks[index].clone())
    }

    fn build_http_response(
        &self,
        certificate: &[u8],
        path: &str,
        encodings: Vec<String>,
        index: usize,
        callback: Func,
        etags: Vec<Hash>,
        req: HttpRequest,
    ) -> HttpResponse {
        let (asset_hash_path, index_hash_path) = if req.get_certificate_version() == 1 {
            let path = AssetPath::from_asset_key(path);
            let v1_path = path.asset_hash_path_v1();

            let index_path = AssetPath::from_asset_key(INDEX_FILE);
            let v1_index = index_path.asset_hash_path_v1();

            (v1_path, v1_index)
        } else {
            let path = AssetPath::from_asset_key(path);
            let v2_root_path = path.asset_hash_path_root_v2();

            let index_path = AssetPath::from_asset_key(INDEX_FILE);
            let v2_index_root = index_path.asset_hash_path_root_v2();

            (v2_root_path, v2_index_root)
        };

        let index_redirect_certificate =
            if self.asset_hashes.get(asset_hash_path.as_vec()).is_none()
                && self.asset_hashes.get(index_hash_path.as_vec()).is_some()
            {
                let absence_proof = self.asset_hashes.witness(asset_hash_path.as_vec());
                let index_proof = self.asset_hashes.witness(index_hash_path.as_vec());
                let combined_proof = merge_hash_trees(absence_proof, index_proof);

                if req.get_certificate_version() == 1 {
                    Some(witness_to_header_v1(combined_proof, certificate))
                } else {
                    Some(witness_to_header_v2(
                        combined_proof,
                        certificate,
                        &asset_hash_path.expr_path(),
                    ))
                }
            } else {
                None
            };

        if let Some(certificate_header) = index_redirect_certificate {
            if let Some(asset) = self.assets.get(INDEX_FILE) {
                if !asset.allow_raw_access() && req.is_raw_domain() {
                    return req.redirect_from_raw_to_certified_domain();
                }
                for enc_name in encodings.iter() {
                    if let Some(enc) = asset.encodings.get(enc_name) {
                        if enc.certified {
                            return HttpResponse::build_ok(
                                asset,
                                enc_name,
                                enc,
                                INDEX_FILE,
                                index,
                                Some(certificate_header),
                                callback,
                                etags,
                            );
                        }
                    }
                }
            }
        }

        let certificate_header = if req.get_certificate_version() == 1 {
            witness_to_header_v1(
                self.asset_hashes.witness(asset_hash_path.as_vec()),
                certificate,
            )
        } else {
            witness_to_header_v2(
                self.asset_hashes.witness(asset_hash_path.as_vec()),
                certificate,
                &asset_hash_path.expr_path(),
            )
        };

        if let Ok(asset) = self.get_asset(&path.into()) {
            if !asset.allow_raw_access() && req.is_raw_domain() {
                return req.redirect_from_raw_to_certified_domain();
            }
            for enc_name in encodings.iter() {
                if let Some(enc) = asset.encodings.get(enc_name) {
                    if enc.certified {
                        return HttpResponse::build_ok(
                            asset,
                            enc_name,
                            enc,
                            path,
                            index,
                            Some(certificate_header),
                            callback,
                            etags,
                        );
                    } else {
                        // Find if identity is certified, if it's not.
                        if let Some(id_enc) = asset.encodings.get("identity") {
                            if id_enc.certified {
                                return HttpResponse::build_ok(
                                    asset,
                                    enc_name,
                                    enc,
                                    path,
                                    index,
                                    Some(certificate_header),
                                    callback,
                                    etags,
                                );
                            }
                        }
                    }
                }
            }
        }

        HttpResponse::build_404(certificate_header)
    }

    pub fn http_request(
        &self,
        req: HttpRequest,
        certificate: &[u8],
        callback: Func,
    ) -> HttpResponse {
        let mut encodings = vec![];
        // waiting for https://dfinity.atlassian.net/browse/BOUN-446
        let etags = Vec::new();
        for (name, value) in req.headers.iter() {
            if name.eq_ignore_ascii_case("Accept-Encoding") {
                for v in value.split(',') {
                    encodings.push(v.trim().to_string());
                }
            }
        }
        encodings.push("identity".to_string());

        let path = match req.url.find('?') {
            Some(i) => &req.url[..i],
            None => &req.url[..],
        };

        match url_decode(path) {
            Ok(path) => {
                self.build_http_response(certificate, &path, encodings, 0, callback, etags, req)
            }
            Err(err) => HttpResponse {
                status_code: 400,
                headers: vec![],
                body: RcBytes::from(ByteBuf::from(format!(
                    "failed to decode path '{}': {}",
                    path, err
                ))),
                streaming_strategy: None,
            },
        }
    }

    pub fn http_request_streaming_callback(
        &self,
        StreamingCallbackToken {
            key,
            content_encoding,
            index,
            sha256,
        }: StreamingCallbackToken,
    ) -> Result<StreamingCallbackHttpResponse, String> {
        let asset = self
            .get_asset(&key)
            .map_err(|_| "Invalid token on streaming: key not found.".to_string())?;
        let enc = asset
            .encodings
            .get(&content_encoding)
            .ok_or_else(|| "Invalid token on streaming: encoding not found.".to_string())?;

        if let Some(expected_hash) = sha256 {
            if expected_hash != enc.sha256 {
                return Err("sha256 mismatch".to_string());
            }
        }

        // MAX is good enough. This means a chunk would be above 64-bits, which is impossible...
        let chunk_index = index.0.to_usize().unwrap_or(usize::MAX);

        Ok(StreamingCallbackHttpResponse {
            body: enc.content_chunks[chunk_index].clone(),
            token: StreamingCallbackToken::create_token(
                &content_encoding,
                enc.content_chunks.len(),
                enc.sha256,
                &key,
                chunk_index,
            ),
        })
    }

    pub fn get_asset_properties(&self, key: AssetKey) -> Result<AssetProperties, String> {
        let asset = self
            .assets
            .get(&key)
            .ok_or_else(|| "asset not found".to_string())?;

        Ok(AssetProperties {
            max_age: asset.max_age,
            headers: asset.headers.clone(),
            allow_raw_access: asset.allow_raw_access,
        })
    }

    pub fn set_asset_properties(&mut self, arg: SetAssetPropertiesArguments) -> Result<(), String> {
        let dependent_keys = self.dependent_keys_v1(&arg.key).clone();
        let asset = self
            .assets
            .get_mut(&arg.key)
            .ok_or_else(|| "asset not found".to_string())?;

        if let Some(headers) = arg.headers {
            asset.headers = headers
        }
        if let Some(max_age) = arg.max_age {
            asset.max_age = max_age
        }
        if let Some(allow_raw_access) = arg.allow_raw_access {
            asset.allow_raw_access = allow_raw_access
        }

        on_asset_change(&mut self.asset_hashes, &arg.key, asset, dependent_keys);
        Ok(())
    }

    // Returns keys that needs to be updated if the supplied key is changed.
    fn dependent_keys_v1<'a>(&self, key: &AssetKey) -> Vec<AssetKey> {
        if self
            .assets
            .get(key)
            .and_then(|asset| asset.is_aliased)
            .unwrap_or(DEFAULT_ALIAS_ENABLED)
        {
            aliased_by_v1(key)
                .into_iter()
                .filter(|k| !self.assets.contains_key(k))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl From<State> for StableState {
    fn from(state: State) -> Self {
        Self {
            authorized: state.authorized,
            stable_assets: state.assets,
        }
    }
}

impl From<StableState> for State {
    fn from(stable_state: StableState) -> Self {
        let mut state = Self {
            authorized: stable_state.authorized,
            assets: stable_state.stable_assets,
            ..Self::default()
        };

        let assets_keys: Vec<_> = state.assets.keys().cloned().collect();
        for key in assets_keys {
            let dependent_keys = state.dependent_keys_v1(&key);
            if let Some(asset) = state.assets.get_mut(&key) {
                for enc in asset.encodings.values_mut() {
                    enc.certified = false;
                }
                on_asset_change(&mut state.asset_hashes, &key, asset, dependent_keys);
            } else {
                // shouldn't reach this
            }
        }
        state
    }
}

fn build_headers<'a>(
    custom_headers: Option<impl Iterator<Item = (impl Into<String>, impl Into<String>)>>,
    max_age: &Option<u64>,
    content_type: impl Into<String>,
    encoding_name: impl Into<String>,
    cert_expr: Option<impl Into<String>>,
) -> HashMap<String, String> {
    let mut headers = HashMap::from([("content-type".to_string(), content_type.into())]);
    if let Some(max_age) = max_age {
        headers.insert("cache-control".to_string(), format!("max-age={}", max_age));
    }
    let encoding_name = encoding_name.into();
    if encoding_name != "identity" {
        headers.insert("content-encoding".to_string(), encoding_name);
    }
    if let Some(arg_headers) = custom_headers {
        for (k, v) in arg_headers {
            headers.insert(k.into().to_lowercase(), v.into());
        }
    }
    if let Some(expr) = cert_expr {
        headers.insert("ic-certificateexpression".to_string(), expr.into());
    }
    headers
}

fn on_asset_change(
    asset_hashes: &mut AssetHashes,
    key: &str,
    asset: &mut Asset,
    dependent_keys: Vec<AssetKey>,
) {
    // update IC-CertificateExpression header value
    asset.update_ic_certificate_expressions();

    // If the most preferred encoding is present and certified,
    // there is nothing to do.
    for enc_name in ENCODING_CERTIFICATION_ORDER.iter() {
        if let Some(enc) = asset.encodings.get(*enc_name) {
            if enc.certified {
                return;
            } else {
                break;
            }
        }
    }

    // Clean up pre-existing paths for this asset
    let mut keys_to_remove = dependent_keys.clone();
    keys_to_remove.push(key.to_string());
    for key in keys_to_remove {
        let key_path = AssetPath::from_asset_key(&key);
        asset_hashes.delete(key_path.asset_hash_path_root_v2().as_vec());
    }
    if asset.encodings.is_empty() {
        return;
    }

    // An encoding with a higher priority was added, let's certify it
    // instead.

    for enc in asset.encodings.values_mut() {
        enc.certified = false;
    }

    // Order of encodings: Follow ENCODING_CERTIFICATION_ORDER,
    // if none exist, we just pick the first existing encoding.
    let mut encoding_order: Vec<String> = ENCODING_CERTIFICATION_ORDER
        .iter()
        .map(|enc| enc.to_string())
        .collect();
    if let Some(enc) = asset.encodings.keys().next() {
        encoding_order.push(enc.clone());
    }

    // Once v1 certification support removed: can move this to just before `enc.certified = true;` happens
    let mut keys_to_insert_hash_for = dependent_keys;
    keys_to_insert_hash_for.push(key.into());
    let keys_to_insert_hash_for: Vec<_> = keys_to_insert_hash_for
        .into_iter()
        .map(|key| {
            let v1_hash_path = AssetPath::from_asset_key(&key).asset_hash_path_v1();
            (key, v1_hash_path)
        })
        .collect();

    // Insert certificate values into hash_tree
    for enc_name in encoding_order.iter() {
        let Asset {
            content_type,
            encodings,
            max_age,
            headers,
            ..
        } = asset;
        if let Some(enc) = encodings.get_mut(enc_name) {
            let mut encoding_headers: Vec<(String, Value)> = build_headers(
                headers.as_ref().map(|h| h.iter()),
                max_age,
                &*content_type,
                enc_name,
                enc.ic_ce.as_ref().map(|ce| &ce.ic_certificate_expression),
            )
            .into_iter()
            .map(|(k, v)| (k, Value::String(v)))
            .collect();
            encoding_headers.push((
                ":ic-cert-status".to_string(),
                Value::String(200.to_string()),
            )); //todo replace with nice version that also precomputes 304 etag responses
            let header_hash = representation_independent_hash(&encoding_headers);
            let response_hash =
                sha2::Sha256::digest(&[header_hash.as_ref(), enc.sha256.as_ref()].concat()).into();
            enc.response_hash = Some(response_hash);

            for (key, v1_path) in keys_to_insert_hash_for {
                let key_path = AssetPath::from_asset_key(&key);
                asset_hashes.insert(v1_path.as_vec(), enc.sha256.into());
                if let Some(hash_path) = enc.asset_hash_path_v2(key_path) {
                    asset_hashes.insert(hash_path.as_vec(), Vec::new());
                }
            }
            enc.certified = true;
            return;
        }
    }
}

// path like /path/to/my/asset should also be valid for /path/to/my/asset.html or /path/to/my/asset/index.html
fn aliases_of_key(key: &AssetKey) -> Vec<AssetKey> {
    if key.ends_with('/') {
        vec![format!("{}index.html", key)]
    } else if !key.ends_with(".html") {
        vec![format!("{}.html", key), format!("{}/index.html", key)]
    } else {
        Vec::new()
    }
}

// Determines possible original keys in case the supplied key is being aliaseded to.
// Sort-of a reverse operation of `alias_of`
fn aliased_by_v1(key: &AssetKey) -> Vec<AssetKey> {
    if key == "/index.html" {
        vec![
            key[..(key.len() - 5)].into(),
            key[..(key.len() - 10)].into(),
        ]
    } else if key.ends_with("/index.html") {
        vec![
            key[..(key.len() - 5)].into(),
            key[..(key.len() - 10)].into(),
            key[..(key.len() - 11)].to_string(),
        ]
    } else if key.ends_with(".html") {
        vec![key[..(key.len() - 5)].to_string()]
    } else {
        Vec::new()
    }
}

pub fn witness_to_header_v1(witness: HashTree, certificate: &[u8]) -> HeaderField {
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    witness.serialize(&mut serializer).unwrap();
    (
        "IC-Certificate".to_string(),
        String::from("certificate=:")
            + &base64::encode(certificate)
            + ":, tree=:"
            + &base64::encode(&serializer.into_inner())
            + ":",
    )
}

pub fn witness_to_header_v2(witness: HashTree, certificate: &[u8], expr_path: &str) -> HeaderField {
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    println!("Witness: {:?}", &witness);
    witness.serialize(&mut serializer).unwrap();

    (
        "IC-Certificate".to_string(),
        String::from("version=2, ")
            + "certificate=:"
            + &base64::encode(certificate)
            + ":, tree=:"
            + &base64::encode(&serializer.into_inner())
            + ":, expr_path=:"
            + expr_path
            + ":",
    )
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
