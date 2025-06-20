use thiserror::Error;

/// Errors encountered while setting the encodings.
#[derive(Error, Debug)]
pub enum SetEncodingError {
    /// Failed when attempting to translate an uploader chunk id to a canister chunk id because the id is unknown.
    #[error("Unknown uploader chunk id: {0}")]
    UnknownUploaderChunkId(usize),
}
