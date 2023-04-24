use self::internals::{
    certification_types::{AssetPath, HashTreePath, NestedTreeKey},
    tree::NestedTree,
};

pub mod internals;
pub use ic_certified_map::HashTree;
pub use ic_response_verification::hash::Value;

pub type CertifiedResponses = NestedTree<NestedTreeKey, Vec<u8>>;

impl CertifiedResponses {
    /// Certifies a response for a number of paths.
    ///
    /// # Arguments
    /// * `paths`: path(s) to the resource
    /// * `status_code`: HTTP status code of the response
    /// * `headers`: All certified headers. It is possible to respond with additional headers, but only the ones supplied in this argument are certified
    /// * `body`: Response body
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
        todo!()
    }

    /// Certifies a response that can be used if no certified response is available for the requested path
    ///
    /// # Arguments
    /// * `status_code`: HTTP status code of the response
    /// * `headers`: All certified headers. It is possible to respond with additional headers, but only the ones supplied in this argument are certified
    /// * `body`: Response body
    /// * `body_hash`: Hash of the response body. If supplied the response body will not be hashed, which can save a lot of computation
    ///
    /// # Return Value
    /// * `HashTreePath`: `HashTreePath` corresponding to the supplied response. Can be used to remove or re-insert certification for this specific response without having to re-compute the full path
    pub fn certify_404_response(
        &mut self,
        status_code: u16,
        headers: &[(String, Value)],
        body: &[u8],
        body_hash: Option<[u8; 32]>,
    ) -> HashTreePath {
        todo!()
    }

    /// Certifies a response. Expects a finished `HashTreePath`, but does not calculate the path from scratch
    pub fn certify_response_precomputed(&mut self, hash_tree_path: &HashTreePath) {
        todo!()
    }

    /// Removes all certified responses for a path
    pub fn remove_responses_for_path(&mut self, path: &str) {
        let key = AssetPath::from(path);
        self.delete(key.asset_hash_path_root_v2().as_vec());
        self.delete(key.asset_hash_path_v1().as_vec());
    }

    /// Removes all certified 404 responses
    pub fn remove_404_responses(&mut self) {
        self.delete(&[
            NestedTreeKey::String("http_expr".into()),
            NestedTreeKey::String("<*>".into()),
        ])
    }

    /// Removes a specific response from the certified responses
    pub fn remove_response_precomputed(&mut self, hash_tree_path: &HashTreePath) {
        todo!()
    }

    /// If the path has certified responses this function creates a hash tree that proves...
    /// * The path is part of the CertifiedResponses hash tree
    /// The hash tree then includes certification for all valid responses for this path.
    ///
    /// If the path has no certified responses this function creates a hash tree that proves...
    /// * The absence of the path in the CertifiedResponses hash tree
    /// * The presence/absence of a 404 response
    /// The hash tree then includes certification for all valid responses for a 404 response.
    pub fn witness_path(&self, path: &str) -> HashTree {
        todo!()
    }

    /// Same as `witness_path`, but produces a header th
    pub fn witness_header(&self, path: &str) -> (String, Value) {
        todo!()
    }
}
