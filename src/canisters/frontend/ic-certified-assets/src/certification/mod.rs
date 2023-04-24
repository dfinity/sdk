use self::internals::{
    certification_types::{AssetPath, HashTreePath, NestedTreeKey},
    tree::NestedTree,
};

use sha2::Digest;

pub mod internals;
pub use ic_certified_map::HashTree;
pub use ic_response_verification::hash::Value;

pub type CertifiedResponses = NestedTree<NestedTreeKey, Vec<u8>>;

impl CertifiedResponses {
    /// Certifies a response for a number of paths with certification v2.
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

    /// Certifies a response for a number of paths with certification v1.
    ///
    /// REPLACES a previously certified response for the given path because v1 certification only supports one certified response per path.
    ///
    /// # Arguments
    /// * `paths`: path(s) to the resource
    /// * `body`: Response body
    /// * `body_hash`: Hash of the response body. If supplied the response body will not be hashed, which can save a lot of computation
    pub fn certify_response_v1(
        &mut self,
        paths: &[&str],
        body: &[u8],
        body_hash: Option<[u8; 32]>,
    ) {
        let body_hash = body_hash.unwrap_or_else(|| sha2::Sha256::digest(body).into());
        let hash_tree_paths: Vec<HashTreePath> = paths
            .into_iter()
            .map(|path| {
                let asset_path = AssetPath::from(path);
                asset_path.asset_hash_path_v1()
            })
            .collect();

        for path in hash_tree_paths.iter() {
            self.insert(path.as_vec(), Vec::from(body_hash));
        }
    }

    /// Certifies a response that can be used if no certified response is available for the requested path with certification v2.
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

    /// Certifies a response. Expects a finished `HashTreePath`, skipping the (sometimes expensive) computation of the `HashTreePath`.
    pub fn certify_response_precomputed(&mut self, path: &HashTreePath) {
        self.insert(path.as_vec(), Vec::new());
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

    /// Removes a specific response from the certified responses. Expects a finished `HashTreePath`, skipping the (sometimes expensive) computation of the `HashTreePath`.
    pub fn remove_response_precomputed(&mut self, path: &HashTreePath) {
        self.delete(path.as_vec());
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
