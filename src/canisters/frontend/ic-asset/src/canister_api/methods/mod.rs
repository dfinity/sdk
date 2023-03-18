pub(crate) mod api_version;
pub(crate) mod batch;
pub(crate) mod chunk;
pub(crate) mod list;

pub(crate) mod method_names {
    pub(crate) const API_VERSION: &str = "api_version";
    pub(crate) const CREATE_BATCH: &str = "create_batch";
    pub(crate) const CREATE_CHUNK: &str = "create_chunk";
    pub(crate) const COMMIT_BATCH: &str = "commit_batch";
    pub(crate) const LIST: &str = "list";
}
