use crate::evidence::EvidenceComputation::{Computed, NextChunkIndex, NextOperation};
use crate::state_machine::{Chunk, State};
use crate::types::BatchOperation::{
    Clear, CreateAsset, DeleteAsset, SetAssetContent, SetAssetProperties, UnsetAssetContent,
};
use crate::types::{
    ChunkId, ClearArguments, CommitBatchArguments, CreateAssetArguments, DeleteAssetArguments,
    SetAssetContentArguments, SetAssetPropertiesArguments, UnsetAssetContentArguments,
};
use itertools::Itertools;
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

const TAG_FALSE: [u8; 1] = [0];
const TAG_TRUE: [u8; 1] = [1];

const TAG_NONE: [u8; 1] = [2];
const TAG_SOME: [u8; 1] = [3];

const TAG_CREATE_ASSET: [u8; 1] = [4];
const TAG_SET_ASSET_CONTENT: [u8; 1] = [5];
const TAG_UNSET_ASSET_CONTENT: [u8; 1] = [6];
const TAG_DELETE_ASSET: [u8; 1] = [7];
const TAG_CLEAR: [u8; 1] = [8];
const TAG_SET_ASSET_PROPERTIES: [u8; 1] = [9];

/// Domain separator and version of the encoding hashed by [`EvidenceComputation`], both for the
/// evidence of a proposed batch and for the state hash.
///
/// Every variable-length field of the encoding is length-prefixed and every operation carries a
/// tag, so each operation is self-delimiting and the encoding is injective over the change a
/// batch applies: its operations, with the content of each `SetAssetContent` taken as one byte
/// string.  It is deliberately *not* injective over `CommitBatchArguments` itself -- two batches
/// that differ only in how they split that content across chunks encode identically.
///
/// The evidence and the state hash share this prefix on purpose: the state hash of an asset
/// canister equals the evidence of the batch that would build it from empty, which is what makes
/// the two comparable.
///
/// The suffix is `v2` because this is the second encoding of this data: the first, which every
/// asset canister installed before API version 3 still computes, carried no separator and no
/// version at all.  So there is no digest anywhere with a `v1` separator, and the number counts
/// encodings rather than separators.  It is not the canister's [`crate::api_version`], which is
/// at 3 and moves independently.  Bump this suffix whenever the encoding changes, so that a hash
/// computed under one version can never equal a hash computed under another.
const ENCODING_DOMAIN: &[u8] = b"ic-certified-assets v2";

pub enum EvidenceComputation {
    NextOperation {
        operation_index: usize,
        hasher: Sha256,
    },

    NextChunkIndex {
        operation_index: usize,
        chunk_index: usize,
        hasher: Sha256,
    },

    Computed(ByteBuf),

    Virtual {
        sorted_keys: Vec<String>,
        current_key_index: usize,
        state: VirtualState,
        hasher: Sha256,
    },
}

#[derive(Clone, Debug)]
pub enum VirtualState {
    CreateAsset,
    SetAssetContent {
        sorted_encoding_names: Vec<String>,
        encoding_index: usize,
    },
    HashChunks {
        sorted_encoding_names: Vec<String>,
        encoding_index: usize,
        chunk_index: usize,
    },
}

impl Default for EvidenceComputation {
    fn default() -> Self {
        Self::new()
    }
}

impl EvidenceComputation {
    pub fn new() -> Self {
        NextOperation {
            operation_index: 0,
            hasher: Sha256::new(),
        }
    }

    pub fn advance(self, args: &CommitBatchArguments, chunks: &HashMap<ChunkId, Chunk>) -> Self {
        match self {
            NextOperation {
                operation_index,
                hasher,
            } => next_operation(args, operation_index, hasher, chunks),
            NextChunkIndex {
                operation_index,
                chunk_index,
                hasher,
            } => next_chunk_index(args, operation_index, chunk_index, hasher, chunks),
            Computed(evidence) => Computed(evidence),
            Self::Virtual { .. } => {
                panic!("Virtual evidence computation cannot be advanced with CommitBatchArguments")
            }
        }
    }

    pub fn advance_virtual(self, state: &crate::state_machine::State) -> Self {
        if let Self::Virtual {
            sorted_keys,
            current_key_index,
            state: virtual_state,
            hasher,
        } = self
        {
            next_virtual_step(state, sorted_keys, current_key_index, virtual_state, hasher)
        } else {
            panic!(
                "EvidenceComputation::advance_virtual called on non-virtual evidence computation"
            );
        }
    }
}

fn next_operation(
    args: &CommitBatchArguments,
    operation_index: usize,
    mut hasher: Sha256,
    chunks: &HashMap<ChunkId, Chunk>,
) -> EvidenceComputation {
    if operation_index == 0 {
        hasher.update(ENCODING_DOMAIN);
    }
    match args.operations.get(operation_index) {
        None => {
            let sha256: [u8; 32] = hasher.finalize().into();
            Computed(ByteBuf::from(sha256))
        }
        Some(CreateAsset(args)) => {
            hash_create_asset(&mut hasher, args);
            NextOperation {
                operation_index: operation_index + 1,
                hasher,
            }
        }
        Some(SetAssetContent(args)) => {
            hash_set_asset_content(&mut hasher, args, set_asset_content_len(args, chunks));
            NextChunkIndex {
                operation_index,
                chunk_index: 0,
                hasher,
            }
        }
        Some(UnsetAssetContent(args)) => {
            hash_unset_asset_content(&mut hasher, args);
            NextOperation {
                operation_index: operation_index + 1,
                hasher,
            }
        }
        Some(DeleteAsset(args)) => {
            hash_delete_asset(&mut hasher, args);
            NextOperation {
                operation_index: operation_index + 1,
                hasher,
            }
        }
        Some(Clear(args)) => {
            hash_clear(&mut hasher, args);
            NextOperation {
                operation_index: operation_index + 1,
                hasher,
            }
        }
        Some(SetAssetProperties(args)) => {
            hash_set_asset_properties(&mut hasher, args);
            NextOperation {
                operation_index: operation_index + 1,
                hasher,
            }
        }
    }
}

fn next_chunk_index(
    args: &CommitBatchArguments,
    operation_index: usize,
    chunk_index: usize,
    mut hasher: Sha256,
    chunks: &HashMap<ChunkId, Chunk>,
) -> EvidenceComputation {
    if let Some(SetAssetContent(sac)) = args.operations.get(operation_index) {
        if let Some(chunk_id) = sac.chunk_ids.get(chunk_index) {
            hash_chunk_by_id(&mut hasher, chunk_id, chunks);
            return NextChunkIndex {
                operation_index,
                chunk_index: chunk_index + 1,
                hasher,
            };
        }
        // `set_asset_content` appends `last_chunk` to the chunks named by `chunk_ids`, so the
        // evidence covers it in the same position and regardless of how many chunk ids precede it.
        if let Some(chunk_content) = sac.last_chunk.as_ref() {
            hash_chunk_by_content(&mut hasher, chunk_content);
        }
    }
    NextOperation {
        operation_index: operation_index + 1,
        hasher,
    }
}

/// The number of content bytes that the evidence covers for a `SetAssetContent` operation, which
/// is exactly the content `set_asset_content` assembles from the same arguments.  Chunk ids that
/// are not present are skipped here and by [`hash_chunk_by_id`] alike.
fn set_asset_content_len(
    args: &SetAssetContentArguments,
    chunks: &HashMap<ChunkId, Chunk>,
) -> usize {
    let from_chunk_ids: usize = args
        .chunk_ids
        .iter()
        .filter_map(|chunk_id| chunks.get(chunk_id))
        .map(|chunk| chunk.content.len())
        .sum();
    from_chunk_ids + args.last_chunk.as_ref().map_or(0, |chunk| chunk.len())
}

fn hash_chunk_by_id(hasher: &mut Sha256, chunk_id: &ChunkId, chunks: &HashMap<ChunkId, Chunk>) {
    if let Some(chunk) = chunks.get(chunk_id) {
        hasher.update(&chunk.content);
    }
}

fn hash_chunk_by_content(hasher: &mut Sha256, chunk_content: &[u8]) {
    hasher.update(chunk_content);
}

fn hash_create_asset(hasher: &mut Sha256, args: &CreateAssetArguments) {
    hasher.update(TAG_CREATE_ASSET);
    hash_str(hasher, &args.key);
    hash_str(hasher, &args.content_type);
    if let Some(max_age) = args.max_age {
        hasher.update(TAG_SOME);
        hasher.update(max_age.to_be_bytes());
    } else {
        hasher.update(TAG_NONE);
    }
    hash_headers(hasher, args.headers.as_ref());
    hash_opt_bool(hasher, args.enable_aliasing);
    hash_opt_bool(hasher, args.allow_raw_access);
}

/// `content_len` is the total number of content bytes hashed after this call, by
/// [`hash_chunk_by_id`] and [`hash_chunk_by_content`].  Hashing it here length-prefixes the
/// content, which makes the operation self-delimiting even though the content bytes themselves
/// are hashed in chunks.
fn hash_set_asset_content(
    hasher: &mut Sha256,
    args: &SetAssetContentArguments,
    content_len: usize,
) {
    hasher.update(TAG_SET_ASSET_CONTENT);
    hash_str(hasher, &args.key);
    hash_str(hasher, &args.content_encoding);
    hash_opt_bytebuf(hasher, args.sha256.as_ref());
    hash_len(hasher, content_len);
}

fn hash_unset_asset_content(hasher: &mut Sha256, args: &UnsetAssetContentArguments) {
    hasher.update(TAG_UNSET_ASSET_CONTENT);
    hash_str(hasher, &args.key);
    hash_str(hasher, &args.content_encoding);
}

fn hash_delete_asset(hasher: &mut Sha256, args: &DeleteAssetArguments) {
    hasher.update(TAG_DELETE_ASSET);
    hash_str(hasher, &args.key);
}

fn hash_clear(hasher: &mut Sha256, _args: &ClearArguments) {
    hasher.update(TAG_CLEAR);
}

fn hash_set_asset_properties(hasher: &mut Sha256, args: &SetAssetPropertiesArguments) {
    hasher.update(TAG_SET_ASSET_PROPERTIES);
    hash_str(hasher, &args.key);
    if let Some(max_age) = args.max_age {
        hasher.update(TAG_SOME);
        if let Some(max_age) = max_age {
            hasher.update(TAG_SOME);
            hasher.update(max_age.to_be_bytes());
        } else {
            hasher.update(TAG_NONE);
        }
    } else {
        hasher.update(TAG_NONE);
    }
    if let Some(headers) = args.headers.as_ref() {
        hasher.update(TAG_SOME);
        hash_headers(hasher, headers.as_ref());
    } else {
        hasher.update(TAG_NONE);
    }
    if let Some(allow_raw_access) = args.allow_raw_access {
        hasher.update(TAG_SOME);
        hash_opt_bool(hasher, allow_raw_access);
    } else {
        hasher.update(TAG_NONE);
    }
    if let Some(enable_aliasing) = args.is_aliased {
        hasher.update(TAG_SOME);
        hash_opt_bool(hasher, enable_aliasing);
    } else {
        hasher.update(TAG_NONE);
    }
}

/// Hashes the length of a repeated or variable-length field, so that the encoding of the field
/// cannot be confused with the encoding of a shorter or longer one.
fn hash_len(hasher: &mut Sha256, len: usize) {
    hasher.update((len as u64).to_be_bytes());
}

/// Hashes a variable-length byte string, prefixed with its length, so that the boundaries of the
/// string are part of the encoding.
fn hash_bytes(hasher: &mut Sha256, bytes: &[u8]) {
    hash_len(hasher, bytes.len());
    hasher.update(bytes);
}

/// Hashes a variable-length text field, prefixed with its length.
fn hash_str(hasher: &mut Sha256, s: &str) {
    hash_bytes(hasher, s.as_bytes());
}

fn hash_opt_bool(hasher: &mut Sha256, b: Option<bool>) {
    if let Some(b) = b {
        hasher.update(TAG_SOME);
        hasher.update(if b { TAG_TRUE } else { TAG_FALSE });
    } else {
        hasher.update(TAG_NONE);
    }
}

fn hash_opt_bytebuf(hasher: &mut Sha256, buf: Option<&ByteBuf>) {
    if let Some(buf) = buf {
        hasher.update(TAG_SOME);
        hash_bytes(hasher, buf);
    } else {
        hasher.update(TAG_NONE);
    }
}

fn hash_headers(hasher: &mut Sha256, headers: Option<&BTreeMap<String, String>>) {
    if let Some(headers) = headers {
        hasher.update(TAG_SOME);
        hash_len(hasher, headers.len());
        for k in headers.keys().sorted() {
            let v = headers.get(k).unwrap();
            hash_str(hasher, k);
            hash_str(hasher, v);
        }
    } else {
        hasher.update(TAG_NONE);
    }
}

#[test]
fn tag_value_uniqueness() {
    let tags = include_str!("evidence.rs")
        .lines()
        .filter(|l| l.starts_with("const TAG_"))
        .map(|line| {
            line.split(": [u8; 1] = [")
                .nth(1)
                .unwrap()
                .trim_end_matches("];")
                .parse::<u8>()
                .unwrap()
        });
    assert_eq!(
        tags.clone().count(),
        tags.unique().count(),
        "tag values must be unique"
    );
}

fn next_virtual_step(
    state: &State,
    sorted_keys: Vec<String>,
    current_key_index: usize,
    virtual_state: VirtualState,
    mut hasher: Sha256,
) -> EvidenceComputation {
    if current_key_index == 0 && matches!(virtual_state, VirtualState::CreateAsset) {
        hasher.update(ENCODING_DOMAIN);
    }

    if current_key_index >= sorted_keys.len() {
        let sha256: [u8; 32] = hasher.finalize().into();
        return EvidenceComputation::Computed(ByteBuf::from(sha256));
    }

    let key = &sorted_keys[current_key_index];
    let asset = state.assets.get(key).expect("asset must exist");

    match virtual_state {
        VirtualState::CreateAsset => {
            let args = CreateAssetArguments {
                key: key.clone(),
                content_type: asset.content_type.clone(),
                max_age: asset.max_age,
                headers: asset.headers.clone(),
                enable_aliasing: asset.is_aliased,
                allow_raw_access: asset.allow_raw_access,
            };
            hash_create_asset(&mut hasher, &args);
            let mut sorted_encoding_names: Vec<String> = asset.encodings.keys().cloned().collect();
            sorted_encoding_names.sort();

            EvidenceComputation::Virtual {
                sorted_keys,
                current_key_index,
                state: VirtualState::SetAssetContent {
                    sorted_encoding_names,
                    encoding_index: 0,
                },
                hasher,
            }
        }
        VirtualState::SetAssetContent {
            sorted_encoding_names,
            encoding_index,
        } => {
            if encoding_index >= sorted_encoding_names.len() {
                // Done with this asset, move to next
                return EvidenceComputation::Virtual {
                    sorted_keys,
                    current_key_index: current_key_index + 1,
                    state: VirtualState::CreateAsset,
                    hasher,
                };
            }

            let enc_name = &sorted_encoding_names[encoding_index];
            let enc = asset.encodings.get(enc_name).expect("encoding must exist");

            let args = SetAssetContentArguments {
                key: key.clone(),
                content_encoding: enc_name.clone(),
                chunk_ids: vec![],
                last_chunk: None,
                sha256: Some(ByteBuf::from(enc.sha256)),
            };
            let content_len = enc
                .content_chunks
                .iter()
                .map(|chunk| chunk.len())
                .sum::<usize>();
            hash_set_asset_content(&mut hasher, &args, content_len);

            EvidenceComputation::Virtual {
                sorted_keys,
                current_key_index,
                state: VirtualState::HashChunks {
                    sorted_encoding_names,
                    encoding_index,
                    chunk_index: 0,
                },
                hasher,
            }
        }
        VirtualState::HashChunks {
            sorted_encoding_names,
            encoding_index,
            chunk_index,
        } => {
            let enc_name = &sorted_encoding_names[encoding_index];
            let enc = asset.encodings.get(enc_name).expect("encoding must exist");

            if chunk_index < enc.content_chunks.len() {
                hash_chunk_by_content(&mut hasher, &enc.content_chunks[chunk_index]);

                EvidenceComputation::Virtual {
                    sorted_keys,
                    current_key_index,
                    state: VirtualState::HashChunks {
                        sorted_encoding_names,
                        encoding_index,
                        chunk_index: chunk_index + 1,
                    },
                    hasher,
                }
            } else {
                // Done with chunks, move to next encoding
                EvidenceComputation::Virtual {
                    sorted_keys,
                    current_key_index,
                    state: VirtualState::SetAssetContent {
                        sorted_encoding_names,
                        encoding_index: encoding_index + 1,
                    },
                    hasher,
                }
            }
        }
    }
}
