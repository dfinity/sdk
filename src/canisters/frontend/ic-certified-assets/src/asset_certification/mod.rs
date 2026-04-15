use self::{
    tree::NestedTree,
    types::{
        certification::{AssetPath, HashTreePath, NestedTreeKey, RequestHash, WitnessResult},
        http::{HeaderField, build_ic_certificate_expression_from_headers, response_hash},
    },
};
use crate::asset_certification::types::http::build_ic_certificate_expression_header;
use ic_certification::merge_hash_trees;
use ic_representation_independent_hash::Value;
use serde::Serialize;
use sha2::Digest;

pub mod tree;
pub mod types;
pub use ic_certification::HashTree;

pub type CertifiedResponses = NestedTree<NestedTreeKey, Vec<u8>>;

impl CertifiedResponses {
    /// Certifies a response for a number of paths with certification v2.
    ///
    /// # Arguments
    /// * `paths`: path(s) to the resource
    /// * `status_code`: HTTP status code of the response
    /// * `headers`: All certified headers. It is possible to respond with additional headers, but only the ones supplied in this argument are certified
    /// * `body`: Response body. Ignored if `body_hash.is_some()`
    /// * `body_hash`: Hash of the response body. If supplied the response body will not be hashed, which can save a lot of computation
    ///
    /// # Return Value
    /// * `Vec<HashTreePath>`: `HashTreePath`s corresponding to the supplied `paths`. Can be used to remove or re-insert certification for a specific response without having to re-compute the full path
    pub fn certify_response(
        &mut self,
        paths: &[&str],
        status_code: u16,
        headers: &[(String, Value)],
        body: &[u8],
        body_hash: Option<[u8; 32]>,
    ) -> Vec<HashTreePath> {
        let certificate_expression = build_ic_certificate_expression_from_headers(headers);
        let request_hash = RequestHash::default(); // request certification currently not supported
        let body_hash = body_hash.unwrap_or_else(|| sha2::Sha256::digest(body).into());
        let response_hash = response_hash(headers, status_code, &body_hash);

        paths
            .iter()
            .map(|path| {
                let asset_path = AssetPath::from(path);
                let hash_tree_path = asset_path.hash_tree_path(
                    &certificate_expression,
                    &request_hash,
                    response_hash,
                );
                self.certify_response_precomputed(&hash_tree_path);
                hash_tree_path
            })
            .collect()
    }

    /// Certifies a response that can be used if no certified response is available for the requested path with certification v2.
    ///
    /// # Arguments
    /// * `status_code`: HTTP status code of the response
    /// * `headers`: All certified headers. It is possible to respond with additional headers, but only the ones supplied in this argument are certified
    /// * `body`: Response body. Ignored if `body_hash.is_some()`
    /// * `body_hash`: Hash of the response body. If supplied the response body will not be hashed, which can save a lot of computation
    ///
    /// # Return Value
    /// * `HashTreePath`: `HashTreePath` corresponding to the supplied response. Can be used to remove or re-insert certification for this specific response without having to re-compute the full path
    pub fn certify_fallback_response(
        &mut self,
        status_code: u16,
        headers: &[(String, Value)],
        body: &[u8],
        body_hash: Option<[u8; 32]>,
    ) -> HashTreePath {
        let certificate_expression = build_ic_certificate_expression_from_headers(headers);
        let cert_expr_header = build_ic_certificate_expression_header(&certificate_expression);
        let cert_expr_header = (cert_expr_header.0, Value::String(cert_expr_header.1));
        let mut certified_headers = Vec::from(headers);
        certified_headers.push(cert_expr_header);
        let request_hash = RequestHash::default(); // request certification currently not supported
        let body_hash = body_hash.unwrap_or_else(|| sha2::Sha256::digest(body).into());
        let response_hash = response_hash(&certified_headers, status_code, &body_hash);

        let asset_path = AssetPath::fallback_path_at(&[]);
        let hash_tree_path =
            asset_path.hash_tree_path(&certificate_expression, &request_hash, response_hash);
        self.certify_response_precomputed(&hash_tree_path);
        hash_tree_path
    }

    /// Certifies a response. Expects a finished `HashTreePath`, skipping the (sometimes expensive) computation of the `HashTreePath`.
    pub fn certify_response_precomputed(&mut self, path: &HashTreePath) {
        self.insert(path.as_vec(), Vec::new());
    }

    /// Removes all certified responses for a path for certification v2
    pub fn remove_responses_for_path(&mut self, path: &str) {
        let key = AssetPath::from(path);
        self.delete(key.asset_hash_path_root_v2().as_vec());
    }

    /// Removes all certified fallback responses at the given `<*>` level for certification v2.
    /// Use `&[]` for the root level.
    pub fn remove_fallback_responses(&mut self, dir_segments: &[&str]) {
        self.delete(HashTreePath::not_found_base_path_v2(dir_segments).as_vec());
    }

    /// Removes a specific response from the certified responses. Expects a finished `HashTreePath`, skipping the (sometimes expensive) computation of the `HashTreePath`.
    pub fn remove_response_precomputed(&mut self, path: &HashTreePath) {
        self.delete(path.as_vec());
    }

    /// If the path has certified responses this function creates a hash tree that proves...
    /// * The path is part of the CertifiedResponses hash tree
    ///
    /// The hash tree then includes certification for all valid responses for this path.
    ///
    /// If the path has no certified responses this function creates a hash tree that proves...
    /// * The absence of the path in the CertifiedResponses hash tree
    /// * The presence/absence of fallback responses at any `<*>` level
    ///
    /// # Return Value
    /// `(tree, result)`
    /// * `result`:
    ///   * `WitnessResult::PathFound` if `path` has a certified response.
    ///   * `WitnessResult::FallbackFound` if the path has no certified response, but some `<*>` level has.
    ///   * `WitnessResult::NoneFound` if neither `path` nor any fallback level has a certified response.
    /// * `tree`: The `HashTree` proving the above.
    pub fn witness_path(&self, path: &str) -> (HashTree, WitnessResult) {
        let path = AssetPath::from(path);
        let hash_tree_path_root = path.asset_hash_path_root_v2();
        if self.contains_path(hash_tree_path_root.as_vec()) {
            (
                self.witness(hash_tree_path_root.as_vec()),
                WitnessResult::PathFound,
            )
        } else {
            let absence_proof = self.witness(hash_tree_path_root.as_vec());
            let fallback_paths = hash_tree_path_root.fallback_paths_v2();

            let combined_proof = fallback_paths
                .iter()
                .fold(absence_proof, |accumulator, path| {
                    let new_proof = self.witness(path.as_vec());
                    merge_hash_trees(accumulator, new_proof)
                });

            let fallback_found = fallback_paths
                .iter()
                .any(|p| self.contains_path(p.as_vec()));

            if fallback_found {
                (combined_proof, WitnessResult::FallbackFound)
            } else {
                (combined_proof, WitnessResult::NoneFound)
            }
        }
    }

    /// Returns the `expr_path` for an exact-match path.
    pub fn expr_path(&self, path: &str) -> String {
        let path = AssetPath::from(path);
        path.asset_hash_path_root_v2().expr_path()
    }

    /// Returns the `expr_path` for a fallback at a given directory level.
    pub fn expr_path_for_fallback(dir_segments: &[&str]) -> String {
        HashTreePath::not_found_base_path_v2(dir_segments).expr_path()
    }

    /// Builds the IC-Certificate header from a witness tree, expr_path, and certificate.
    pub fn build_certificate_header(
        witness: &HashTree,
        expr_path: &str,
        certificate: &[u8],
    ) -> HeaderField {
        let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
        serializer.self_describe().unwrap();
        witness.serialize(&mut serializer).unwrap();

        (
            "IC-Certificate".to_string(),
            String::from("version=2, ")
                + "certificate=:"
                + &base64::encode(certificate)
                + ":, tree=:"
                + &base64::encode(serializer.into_inner())
                + ":, expr_path=:"
                + expr_path
                + ":",
        )
    }
}
