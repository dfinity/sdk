//! This module contains a pure implementation of the certified assets state machine.

// NB. This module should not depend on ic_cdk, it contains only pure state transition functions.
// All the environment (time, certificates, etc.) is passed to the state transition functions
// as formal arguments.  This approach makes it very easy to test the state machine.

use crate::{
    certification_types::{
        AssetHashes, AssetPath, CertificateExpression, HashTreePath, NestedTreeKey,
    },
    evidence::{EvidenceComputation, EvidenceComputation::Computed},
    http::{
        build_ic_certificate_expression_from_headers_and_encoding, witness_to_header_v1,
        witness_to_header_v2, HttpRequest, HttpResponse, StreamingCallbackHttpResponse,
        StreamingCallbackToken,
    },
    rc_bytes::RcBytes,
    tree::merge_hash_trees,
    types::*,
    url_decode::url_decode,
};
use candid::{CandidType, Deserialize, Func, Int, Nat, Principal};
use ic_certified_map::{AsHashTree, Hash};
use ic_response_verification::hash::{representation_independent_hash, Value};
use num_traits::ToPrimitive;
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha2::Digest;
use std::collections::{BTreeSet, HashMap};
use std::convert::TryInto;

/// The amount of time a batch is kept alive. Modifying the batch
/// delays the expiry further.
pub const BATCH_EXPIRY_NANOS: u64 = 300_000_000_000;

/// The order in which we pick encodings for certification.
const ENCODING_CERTIFICATION_ORDER: &[&str] = &["identity", "gzip", "compress", "deflate", "br"];
// Order of encodings is relevant for v1. Follow ENCODING_CERTIFICATION_ORDER,
// then follow the order of existing encodings.
// For v2, it is important to certify all encodings, therefore all encodings are added to the list.
pub fn encoding_certification_order<'a>(
    actual_encodings: impl Iterator<Item = &'a String>,
) -> Vec<String> {
    let mut encoding_order: Vec<String> = ENCODING_CERTIFICATION_ORDER
        .iter()
        .map(|enc| enc.to_string())
        .collect();
    encoding_order.append(
        &mut actual_encodings
            .filter(|encoding| !ENCODING_CERTIFICATION_ORDER.contains(&encoding.as_str()))
            .map(|s| s.into())
            .collect(),
    );
    encoding_order
}

/// The file to serve if the requested file wasn't found.
const INDEX_FILE: &str = "/index.html";

/// Default aliasing behavior.
const DEFAULT_ALIAS_ENABLED: bool = true;

const STATUS_CODES_TO_CERTIFY: [u16; 2] = [200, 304];

const DEFAULT_MAX_COMPUTE_EVIDENCE_ITERATIONS: u16 = 20;

type Timestamp = Int;

#[derive(Default, Clone, Debug, CandidType, Deserialize)]
pub struct AssetEncoding {
    pub modified: Timestamp,
    pub content_chunks: Vec<RcBytes>,
    pub total_length: usize,
    /// Valid as-is for v2.
    /// For v1, also make sure that encoding name == asset.most_important_encoding_v1()
    pub certified: bool,
    pub sha256: [u8; 32],
    pub certificate_expression: Option<CertificateExpression>,
    pub response_hashes: Option<HashMap<u16, [u8; 32]>>,
}

impl AssetEncoding {
    fn asset_hash_path_v2(
        &self,
        AssetPath(path): &AssetPath,
        status_code: u16,
    ) -> Option<HashTreePath> {
        self.certificate_expression.as_ref().and_then(|ce| {
            self.response_hashes.as_ref().and_then(|hashes| {
                hashes.get(&status_code).map(|response_hash| {
                    let mut path: Vec<NestedTreeKey> =
                        path.iter().map(|segment| segment.as_str().into()).collect();
                    path.insert(0, "http_expr".into());
                    path.push("<$>".into()); // asset path terminator
                    path.push(ce.hash.as_slice().into());
                    path.push("".into()); // no request certification - use empty node
                    path.push(response_hash.as_slice().into());
                    path.into()
                })
            })
        })
    }

    fn not_found_hash_path(&self) -> Option<HashTreePath> {
        self.certificate_expression.as_ref().and_then(|ce| {
            self.response_hashes
                .as_ref()
                .and_then(|hashes| hashes.get(&200))
                .map(|response_hash| {
                    HashTreePath::from(Vec::<NestedTreeKey>::from([
                        "http_expr".into(),
                        "<*>".into(), // 404 not found wildcard segment
                        ce.hash.as_slice().into(),
                        "".into(), // no request certification - use empty node
                        response_hash.as_slice().into(),
                    ]))
                })
        })
    }

    fn compute_response_hashes(
        &self,
        headers: &Option<HashMap<String, String>>,
        max_age: &Option<u64>,
        content_type: &str,
        encoding_name: &str,
    ) -> HashMap<u16, [u8; 32]> {
        fn compute_response_hash(
            base_headers: &[(String, Value)],
            status_code: u16,
            body_hash: &[u8; 32],
        ) -> [u8; 32] {
            // certification v2 spec:
            // Response hash is the hash of the concatenation of
            //   - representation-independent hash of headers
            //   - hash of the response body
            //
            // The representation-independent hash of headers consist of
            //    - all certified headers (here all headers), plus
            //    - synthetic header `:ic-cert-status` with value <HTTP status code of response>

            let mut headers = Vec::from(base_headers);
            headers.push((
                ":ic-cert-status".to_string(),
                Value::Number(status_code.into()),
            ));
            let header_hash = representation_independent_hash(&headers);
            sha2::Sha256::digest(&[header_hash.as_ref(), body_hash].concat()).into()
        }

        // Collect all user-defined headers
        let base_headers: Vec<(String, Value)> = build_headers(
            headers.as_ref().map(|h| h.iter()),
            max_age,
            content_type,
            encoding_name,
            self.certificate_expression
                .as_ref()
                .map(|ce| &ce.expression),
        )
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();

        // HTTP 200
        let response_hash_200 = compute_response_hash(&base_headers, 200, &self.sha256);

        // HTTP 304
        let empty_body_hash: [u8; 32] = sha2::Sha256::digest([]).into();
        let response_hash_304 = compute_response_hash(&base_headers, 304, &empty_body_hash);

        let mut response_hashes = HashMap::new();
        response_hashes.insert(200, response_hash_200);
        response_hashes.insert(304, response_hash_304);

        debug_assert!(STATUS_CODES_TO_CERTIFY
            .iter()
            .all(|code| response_hashes.contains_key(code)));

        response_hashes
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
    pub commit_batch_arguments: Option<CommitBatchArguments>,
    pub evidence_computation: Option<EvidenceComputation>,
}

#[derive(Default)]
pub struct State {
    assets: HashMap<AssetKey, Asset>,

    chunks: HashMap<ChunkId, Chunk>,
    next_chunk_id: ChunkId,

    batches: HashMap<BatchId, Batch>,
    next_batch_id: BatchId,

    // permissions
    commit_principals: BTreeSet<Principal>,
    prepare_principals: BTreeSet<Principal>,
    manage_permissions_principals: BTreeSet<Principal>,

    asset_hashes: AssetHashes,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableStatePermissions {
    commit: BTreeSet<Principal>,
    prepare: BTreeSet<Principal>,
    manage_permissions: BTreeSet<Principal>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct StableState {
    authorized: Vec<Principal>, // ignored if permissions is Some(_)
    permissions: Option<StableStatePermissions>,
    stable_assets: HashMap<String, Asset>,

    next_batch_id: Option<BatchId>,
}

impl Asset {
    fn allow_raw_access(&self) -> bool {
        self.allow_raw_access.unwrap_or(false)
    }

    fn update_ic_certificate_expressions(&mut self) {
        // gather all headers
        let mut header_names = vec![];

        if self.max_age.is_some() {
            header_names.push("cache-control");
        }
        if let Some(custom_headers) = &self.headers {
            for (k, _) in custom_headers.iter() {
                header_names.push(k);
            }
        }

        // update
        for (enc_name, encoding) in self.encodings.iter_mut() {
            encoding.certificate_expression = Some(
                build_ic_certificate_expression_from_headers_and_encoding(&header_names, enc_name),
            );
        }
    }

    pub fn get_headers_for_asset(
        &self,
        encoding_name: &str,
        cert_version: u16,
    ) -> HashMap<String, String> {
        let ce = if cert_version != 1 {
            self.encodings
                .get(encoding_name)
                .and_then(|e| e.certificate_expression.as_ref().map(|ce| &ce.expression))
        } else {
            None
        };
        build_headers(
            self.headers.as_ref().map(|h| h.iter()),
            &self.max_age,
            &self.content_type,
            encoding_name.to_owned(),
            ce,
        )
    }

    // certification v1 only certifies the most important encoding
    pub fn most_important_encoding_v1(&self) -> String {
        for enc in encoding_certification_order(self.encodings.keys()).into_iter() {
            if self.encodings.contains_key(&enc) {
                return enc;
            }
        }
        "no encoding found".to_string()
    }
}

impl State {
    fn get_asset(&self, key: &AssetKey) -> Result<&Asset, String> {
        self.assets
            .get(key)
            .or_else(|| {
                let aliased = aliases_of(key)
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

    pub fn grant_permission(&mut self, principal: Principal, permission: &Permission) {
        let permitted = self.get_mut_permission_list(permission);
        permitted.insert(principal);
    }

    pub fn revoke_permission(&mut self, principal: Principal, permission: &Permission) {
        let permitted = self.get_mut_permission_list(permission);
        permitted.remove(&principal);
    }

    pub fn list_permitted(&self, permission: &Permission) -> &BTreeSet<Principal> {
        self.get_permission_list(permission)
    }

    pub fn take_ownership(&mut self, controller: Principal) {
        self.commit_principals.clear();
        self.prepare_principals.clear();
        self.manage_permissions_principals.clear();
        self.commit_principals.insert(controller);
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

        let dependent_keys = self.dependent_keys(&arg.key);
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
            certificate_expression: None, // set by on_asset_change
            response_hashes: None,        // set by on_asset_change
        };
        asset.encodings.insert(arg.content_encoding, enc);

        on_asset_change(&mut self.asset_hashes, &arg.key, asset, dependent_keys);

        Ok(())
    }

    pub fn unset_asset_content(&mut self, arg: UnsetAssetContentArguments) -> Result<(), String> {
        let dependent_keys = self.dependent_keys(&arg.key);
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
        if self.assets.contains_key(&arg.key) {
            for dependent in self.dependent_keys(&arg.key) {
                let path = AssetPath::from(dependent);
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

    pub fn has_permission(&self, principal: &Principal, permission: &Permission) -> bool {
        let list = self.get_permission_list(permission);
        list.contains(principal)
    }

    pub fn can(&self, principal: &Principal, permission: &Permission) -> bool {
        self.has_permission(principal, permission)
            || (*permission == Permission::Prepare
                && self.has_permission(principal, &Permission::Commit))
    }

    fn get_permission_list(&self, permission: &Permission) -> &BTreeSet<Principal> {
        match permission {
            Permission::Commit => &self.commit_principals,
            Permission::Prepare => &self.prepare_principals,
            Permission::ManagePermissions => &self.manage_permissions_principals,
        }
    }

    fn get_mut_permission_list(&mut self, permission: &Permission) -> &mut BTreeSet<Principal> {
        match permission {
            Permission::Commit => &mut self.commit_principals,
            Permission::Prepare => &mut self.prepare_principals,
            Permission::ManagePermissions => &mut self.manage_permissions_principals,
        }
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
        let dependent_keys = self.dependent_keys(&arg.key);
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
                commit_batch_arguments: None,
                evidence_computation: None,
            },
        );
        self.chunks.retain(|_, c| {
            self.batches
                .get(&c.batch_id)
                .map(|b| b.expires_at > now || b.commit_batch_arguments.is_some())
                .unwrap_or(false)
        });
        self.batches
            .retain(|_, b| b.expires_at > now || b.commit_batch_arguments.is_some());

        batch_id
    }

    pub fn create_chunk(&mut self, arg: CreateChunkArg, now: u64) -> Result<ChunkId, String> {
        let mut batch = self
            .batches
            .get_mut(&arg.batch_id)
            .ok_or_else(|| "batch not found".to_string())?;
        if batch.commit_batch_arguments.is_some() {
            return Err("batch has been proposed".to_string());
        }

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

    pub fn propose_commit_batch(&mut self, arg: CommitBatchArguments) -> Result<(), String> {
        let batch = self
            .batches
            .get_mut(&arg.batch_id)
            .expect("batch not found");
        if batch.commit_batch_arguments.is_some() {
            return Err("batch already has proposed CommitBatchArguments".to_string());
        };
        batch.commit_batch_arguments = Some(arg);
        Ok(())
    }

    pub fn commit_proposed_batch(
        &mut self,
        arg: CommitProposedBatchArguments,
        now: u64,
    ) -> Result<(), String> {
        self.validate_commit_proposed_batch_args(&arg)?;
        let batch = self.batches.get_mut(&arg.batch_id).unwrap();
        let proposed_batch_arguments = batch.commit_batch_arguments.take().unwrap();
        self.commit_batch(proposed_batch_arguments, now)
    }

    pub fn validate_commit_proposed_batch(
        &self,
        arg: CommitProposedBatchArguments,
    ) -> Result<String, String> {
        self.validate_commit_proposed_batch_args(&arg)?;
        Ok(format!(
            "commit proposed batch {} with evidence {}",
            arg.batch_id,
            hex::encode(arg.evidence)
        ))
    }

    fn validate_commit_proposed_batch_args(
        &self,
        arg: &CommitProposedBatchArguments,
    ) -> Result<(), String> {
        let batch = self.batches.get(&arg.batch_id).ok_or("batch not found")?;
        if batch.commit_batch_arguments.is_none() {
            return Err("batch does not have CommitBatchArguments".to_string());
        };
        let evidence = if let Some(Computed(evidence)) = &batch.evidence_computation {
            evidence.clone()
        } else {
            return Err("batch does not have computed evidence".to_string());
        };
        if evidence != arg.evidence {
            return Err(format!(
                "batch computed evidence {} does not match presented evidence {}",
                hex::encode(evidence),
                hex::encode(&arg.evidence)
            ));
        }
        Ok(())
    }

    pub fn compute_evidence(
        &mut self,
        arg: ComputeEvidenceArguments,
    ) -> Result<Option<ByteBuf>, String> {
        let batch = self
            .batches
            .get_mut(&arg.batch_id)
            .expect("batch not found");

        let cba = batch
            .commit_batch_arguments
            .as_ref()
            .expect("batch does not have CommitBatchArguments");

        let max_iterations = arg
            .max_iterations
            .unwrap_or(DEFAULT_MAX_COMPUTE_EVIDENCE_ITERATIONS);

        let mut ec = batch.evidence_computation.take().unwrap_or_default();
        for _ in 0..max_iterations {
            ec = ec.advance(cba, &self.chunks);
            if matches!(ec, Computed(_)) {
                break;
            }
        }
        batch.evidence_computation = Some(ec);

        if let Some(Computed(evidence)) = &batch.evidence_computation {
            Ok(Some(evidence.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn delete_batch(&mut self, arg: DeleteBatchArguments) -> Result<(), String> {
        if self.batches.remove(&arg.batch_id).is_none() {
            return Err("batch not found".to_string());
        }
        self.chunks.retain(|_, c| c.batch_id != arg.batch_id);
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
        requested_encodings: Vec<String>,
        chunk_index: usize,
        callback: Func,
        etags: Vec<Hash>,
        req: HttpRequest,
    ) -> HttpResponse {
        let (asset_hash_path, not_found_hash_path) = if req.get_certificate_version() == 1 {
            let path = AssetPath::from(path);
            let v1_path = path.asset_hash_path_v1();

            let not_found_path = AssetPath::from(INDEX_FILE);
            let v1_not_found = not_found_path.asset_hash_path_v1();

            (v1_path, v1_not_found)
        } else {
            let path = AssetPath::from(path);
            let v2_root_path = path.asset_hash_path_root_v2();

            let v2_not_found_root = HashTreePath::from(Vec::from([
                NestedTreeKey::String("http_expr".into()),
                NestedTreeKey::String("<*>".into()),
            ]));

            (v2_root_path, v2_not_found_root)
        };

        let index_redirect_certificate =
            if !self.asset_hashes.contains_path(asset_hash_path.as_vec())
                && self
                    .asset_hashes
                    .contains_path(not_found_hash_path.as_vec())
            {
                let absence_proof = self.asset_hashes.witness(asset_hash_path.as_vec());
                let not_found_proof = self.asset_hashes.witness(not_found_hash_path.as_vec());
                let combined_proof = merge_hash_trees(absence_proof, not_found_proof);

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

        if let Some(certificate_header) = index_redirect_certificate.as_ref() {
            if let Ok(asset) = self.get_asset(&INDEX_FILE.to_string()) {
                if !asset.allow_raw_access() && req.is_raw_domain() {
                    return req.redirect_from_raw_to_certified_domain();
                }
                if let Some(response) = HttpResponse::build_ok_from_requested_encodings(
                    asset,
                    &requested_encodings,
                    path,
                    chunk_index,
                    Some(certificate_header),
                    &callback,
                    &etags,
                    req.get_certificate_version(),
                ) {
                    return response;
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
            if let Some(response) = HttpResponse::build_ok_from_requested_encodings(
                asset,
                &requested_encodings,
                path,
                chunk_index,
                Some(&certificate_header),
                &callback,
                &etags,
                req.get_certificate_version(),
            ) {
                return response;
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
            is_aliased: asset.is_aliased,
        })
    }

    pub fn set_asset_properties(&mut self, arg: SetAssetPropertiesArguments) -> Result<(), String> {
        let dependent_keys = self.dependent_keys(&arg.key);
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

        if let Some(is_aliased) = arg.is_aliased {
            asset.is_aliased = is_aliased
        }

        on_asset_change(&mut self.asset_hashes, &arg.key, asset, dependent_keys);

        Ok(())
    }

    // Returns keys that needs to be updated if the supplied key is changed.
    fn dependent_keys(&self, key: &AssetKey) -> Vec<AssetKey> {
        if self
            .assets
            .get(key)
            .and_then(|asset| asset.is_aliased)
            .unwrap_or(DEFAULT_ALIAS_ENABLED)
        {
            aliased_by(key)
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
        let permissions = StableStatePermissions {
            commit: state.commit_principals,
            prepare: state.prepare_principals,
            manage_permissions: state.manage_permissions_principals,
        };
        Self {
            authorized: vec![],
            permissions: Some(permissions),
            stable_assets: state.assets,
            next_batch_id: Some(state.next_batch_id),
        }
    }
}

impl From<StableState> for State {
    fn from(stable_state: StableState) -> Self {
        let (commit_principals, prepare_principals, manage_permissions_principals) =
            if let Some(permissions) = stable_state.permissions {
                (
                    permissions.commit,
                    permissions.prepare,
                    permissions.manage_permissions,
                )
            } else {
                (
                    stable_state.authorized.into_iter().collect(),
                    BTreeSet::new(),
                    BTreeSet::new(),
                )
            };
        let mut state = Self {
            commit_principals,
            prepare_principals,
            manage_permissions_principals,
            assets: stable_state.stable_assets,
            next_batch_id: stable_state.next_batch_id.unwrap_or_else(|| Nat::from(1)),
            ..Self::default()
        };

        let assets_keys: Vec<_> = state.assets.keys().cloned().collect();
        for key in assets_keys {
            let dependent_keys = state.dependent_keys(&key);
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

fn build_headers(
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
    let mut affected_keys = dependent_keys;
    affected_keys.push(key.to_string());

    delete_preexisting_asset_hashes(asset_hashes, &affected_keys);

    if asset.encodings.is_empty() {
        return;
    }

    for enc in asset.encodings.values_mut() {
        enc.certified = false;
    }

    asset.update_ic_certificate_expressions();

    let most_important_encoding_v1 = asset.most_important_encoding_v1();
    let Asset {
        content_type,
        encodings,
        max_age,
        headers,
        ..
    } = asset;
    // Insert certified response values into hash_tree
    // Once certification v1 support is removed, encoding_certification_order().iter() can be replaced with asset.encodings.iter_mut()
    for enc_name in encoding_certification_order(encodings.keys()).iter() {
        if let Some(enc) = encodings.get_mut(enc_name) {
            enc.response_hashes =
                Some(enc.compute_response_hashes(headers, max_age, content_type, enc_name));

            insert_new_response_hashes_for_encoding(
                asset_hashes,
                enc,
                &affected_keys,
                enc_name == &most_important_encoding_v1,
            );
            enc.certified = true;
        }
    }
}

fn delete_preexisting_asset_hashes(asset_hashes: &mut AssetHashes, affected_keys: &[String]) {
    for key in affected_keys.iter() {
        let key_path = AssetPath::from(key);
        asset_hashes.delete(key_path.asset_hash_path_root_v2().as_vec());
        asset_hashes.delete(key_path.asset_hash_path_v1().as_vec());
        if key == INDEX_FILE {
            asset_hashes.delete(&[
                NestedTreeKey::String("http_expr".into()),
                NestedTreeKey::String("<*>".into()),
            ]);
        }
    }
}

fn insert_new_response_hashes_for_encoding(
    asset_hashes: &mut AssetHashes,
    enc: &AssetEncoding,
    affected_keys: &Vec<String>,
    is_most_important_encoding: bool,
) {
    for key in affected_keys {
        let key_path = AssetPath::from(&key);
        let v1_path = key_path.asset_hash_path_v1();
        if is_most_important_encoding {
            // v1 can only certify one encoding, therefore we only certify the most important one
            asset_hashes.insert(v1_path.as_vec(), enc.sha256.into());
        }
        for status_code in STATUS_CODES_TO_CERTIFY {
            if let Some(hash_path) = enc.asset_hash_path_v2(&key_path, status_code) {
                asset_hashes.insert(hash_path.as_vec(), Vec::new());
            } else {
                unreachable!(
                    "Could not create a hash path for a status code {} and key {} - did you forget to compute a response hash for this status code?",
                    status_code, &key
                );
            }
        }
        if key == INDEX_FILE {
            if let Some(not_found_hash_path) = enc.not_found_hash_path() {
                asset_hashes.insert(not_found_hash_path.as_vec(), Vec::new());
            }
        }
    }
}

// path like /path/to/my/asset should also be valid for /path/to/my/asset.html or /path/to/my/asset/index.html
fn aliases_of(key: &AssetKey) -> Vec<AssetKey> {
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
fn aliased_by(key: &AssetKey) -> Vec<AssetKey> {
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
