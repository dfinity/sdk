use crate::evidence::EvidenceComputation::{Computed, NextChunkIndex, NextOperation};
use crate::state_machine::Chunk;
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
use std::collections::HashMap;

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
            } => next_operation(args, operation_index, hasher),
            NextChunkIndex {
                operation_index,
                chunk_index,
                hasher,
            } => next_chunk_index(args, operation_index, chunk_index, hasher, chunks),
            Computed(evidence) => Computed(evidence),
        }
    }
}

fn next_operation(
    args: &CommitBatchArguments,
    operation_index: usize,
    mut hasher: Sha256,
) -> EvidenceComputation {
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
            hash_set_asset_content(&mut hasher, args);
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
            if chunk_index + 1 < sac.chunk_ids.len() {
                return NextChunkIndex {
                    operation_index,
                    chunk_index: chunk_index + 1,
                    hasher,
                };
            }
        }
    }
    NextOperation {
        operation_index: operation_index + 1,
        hasher,
    }
}

fn hash_chunk_by_id(hasher: &mut Sha256, chunk_id: &ChunkId, chunks: &HashMap<ChunkId, Chunk>) {
    if let Some(chunk) = chunks.get(chunk_id) {
        hasher.update(&chunk.content);
    }
}

fn hash_create_asset(hasher: &mut Sha256, args: &CreateAssetArguments) {
    hasher.update(TAG_CREATE_ASSET);
    hasher.update(&args.key);
    hasher.update(&args.content_type);
    if let Some(max_age) = args.max_age {
        hasher.update(TAG_SOME);
        hasher.update(max_age.to_be_bytes());
    } else {
        hasher.update(TAG_NONE);
    }
    hash_headers(hasher, args.headers.as_ref());
    hash_opt_bool(hasher, args.allow_raw_access);
    hash_opt_bool(hasher, args.enable_aliasing);
}

fn hash_set_asset_content(hasher: &mut Sha256, args: &SetAssetContentArguments) {
    hasher.update(TAG_SET_ASSET_CONTENT);
    hasher.update(&args.key);
    hasher.update(&args.content_encoding);
    hash_opt_bytebuf(hasher, args.sha256.as_ref());
}

fn hash_unset_asset_content(hasher: &mut Sha256, args: &UnsetAssetContentArguments) {
    hasher.update(TAG_UNSET_ASSET_CONTENT);
    hasher.update(&args.key);
    hasher.update(&args.content_encoding);
}

fn hash_delete_asset(hasher: &mut Sha256, args: &DeleteAssetArguments) {
    hasher.update(TAG_DELETE_ASSET);
    hasher.update(&args.key);
}

fn hash_clear(hasher: &mut Sha256, _args: &ClearArguments) {
    hasher.update(TAG_CLEAR);
}

fn hash_set_asset_properties(hasher: &mut Sha256, args: &SetAssetPropertiesArguments) {
    hasher.update(TAG_SET_ASSET_PROPERTIES);
    hasher.update(&args.key);
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
        hasher.update(buf);
    } else {
        hasher.update(TAG_NONE);
    }
}

fn hash_headers(hasher: &mut Sha256, headers: Option<&HashMap<String, String>>) {
    if let Some(headers) = headers {
        hasher.update(TAG_SOME);
        for k in headers.keys().sorted() {
            let v = headers.get(k).unwrap();
            hasher.update(k);
            hasher.update(v);
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
