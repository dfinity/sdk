use crate::AssetSyncProgressRenderer;
use crate::asset::content::Content;
use crate::asset::content_encoder::ContentEncoder::{self, Brotli, Gzip};
use crate::batch_upload::operations::AssetDeletionReason::Obsolete;
use crate::batch_upload::operations::assemble_batch_operations;
use crate::batch_upload::plumbing::{MAX_CHUNK_SIZE, ProjectAsset, make_project_assets};
use crate::canister_api::methods::api_version::api_version;
use crate::canister_api::methods::asset_properties::get_assets_properties;
use crate::canister_api::methods::list::list_assets;
use crate::canister_api::types::asset::SetAssetPropertiesArguments;
use crate::canister_api::types::batch_upload::common::{
    ClearArguments, CreateAssetArguments, DeleteAssetArguments, SetAssetContentArguments,
    UnsetAssetContentArguments,
};
use crate::canister_api::types::batch_upload::v1::BatchOperationKind;
use crate::error::ComputeEvidenceError;
use crate::error::HashContentError;
use crate::error::HashContentError::EncodeContentFailed;
use crate::error::{SyncError, UploadContentError};
use crate::sync::gather_asset_descriptors;
use ic_utils::Canister;
use mime::Mime;
use sha2::{Digest, Sha256};
use slog::{Logger, info, trace};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

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

/// Domain separator and version of the encoding hashed for the evidence of a proposed batch and
/// for the state hash.  Every variable-length field is length-prefixed and every operation
/// carries a tag, so each operation is self-delimiting and the encoding is an injective function
/// of the operations it is computed over.
///
/// Must match `ENCODING_DOMAIN` in `ic-certified-assets`, as must the rest of the encoding: the
/// point of computing these hashes here is to compare them with the values the asset canister
/// computes.
const ENCODING_DOMAIN: &[u8] = b"ic-certified-assets v2";

/// The lowest asset canister API version that computes evidence with the encoding above.  An
/// asset canister reporting less than this computes a different value over the same batch, so
/// there is nothing to compare against and we say so instead of returning a digest that will not
/// match.
pub(crate) const EVIDENCE_API_VERSION: u16 = 3;

/// Compute the hash ("evidence") over the batch operations required to update the assets
pub async fn compute_evidence(
    canister: &Canister<'_>,
    dirs: &[&Path],
    logger: &Logger,
    progress: Option<&dyn AssetSyncProgressRenderer>,
) -> Result<String, ComputeEvidenceError> {
    let canister_api_version = api_version(canister)
        .await
        .map_err(ComputeEvidenceError::ApiVersionQueryFailed)?;
    if canister_api_version < EVIDENCE_API_VERSION {
        return Err(ComputeEvidenceError::EvidenceApiVersionTooLow {
            canister_api_version,
            required_api_version: EVIDENCE_API_VERSION,
        });
    }

    let asset_descriptors = gather_asset_descriptors(dirs, logger)?;

    let canister_assets = list_assets(canister)
        .await
        .map_err(ComputeEvidenceError::ListAssets)?;
    info!(
        logger,
        "Fetching properties for all assets in the canister."
    );
    let canister_asset_properties =
        get_assets_properties(canister, &canister_assets, progress).await?;

    info!(
        logger,
        "Computing evidence for batch operations for assets in the project.",
    );

    let project_assets = make_project_assets(
        None,
        asset_descriptors,
        &canister_assets,
        crate::batch_upload::plumbing::Mode::ByProposal,
        logger,
        progress,
    )
    .await?;

    let mut operations = assemble_batch_operations(
        None,
        &project_assets,
        canister_assets,
        Obsolete,
        canister_asset_properties,
    )
    .await
    .map_err(ComputeEvidenceError::AssembleCommitBatchArgumentFailed)?;
    operations.sort();
    trace!(logger, "{:#?}", operations);

    let mut sha = Sha256::new();
    sha.update(ENCODING_DOMAIN);
    for op in operations {
        hash_operation(&mut sha, &op, &project_assets)?;
    }
    let evidence: [u8; 32] = sha.finalize().into();

    Ok(hex::encode(evidence))
}

/// Locally computes the state hash of the asset canister if it were synchronized with the given directories.
#[allow(clippy::result_large_err)]
pub fn compute_state_hash(dirs: &[&Path], logger: &Logger) -> Result<String, SyncError> {
    let asset_descriptors = gather_asset_descriptors(dirs, logger)
        .map_err(UploadContentError::GatherAssetDescriptorsFailed)
        .map_err(SyncError::UploadContentFailed)?;
    let mut sorted_asset_descriptors = asset_descriptors;
    sorted_asset_descriptors.sort_by(|a, b| a.key.cmp(&b.key));

    let mut hasher = Sha256::new();
    hasher.update(ENCODING_DOMAIN);

    for asset in sorted_asset_descriptors {
        let content = Content::load(&asset.source).map_err(|e| {
            SyncError::UploadContentFailed(UploadContentError::CreateProjectAssetError(
                crate::error::CreateProjectAssetError::LoadContentFailed(e),
            ))
        })?;

        let create_args = CreateAssetArguments {
            key: asset.key.clone(),
            content_type: content.media_type.to_string(),
            max_age: asset.config.cache.as_ref().and_then(|c| c.max_age),
            headers: asset.config.combined_headers(),
            enable_aliasing: asset.config.enable_aliasing,
            allow_raw_access: asset.config.allow_raw_access,
        };
        hash_create_asset(&mut hasher, &create_args);

        let encoders = asset
            .config
            .encodings
            .clone()
            .unwrap_or_else(|| default_encoders(&content.media_type));
        let force_encoding = !encoders.contains(&ContentEncoder::Identity);

        let mut encodings = Vec::new();

        for encoder in encoders {
            if let Ok(encoded) = content.encode(&encoder) {
                if encoder == ContentEncoder::Identity
                    || force_encoding
                    || encoded.data.len() < content.data.len()
                {
                    encodings.push((encoder, encoded));
                }
            }
        }

        encodings.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

        for (encoder, encoded_content) in encodings {
            let sha256 = encoded_content.sha256();
            let set_content_args = SetAssetContentArguments {
                key: asset.key.clone(),
                content_encoding: encoder.to_string(),
                chunk_ids: vec![], // ignored by hash_set_asset_content
                last_chunk: None,  // ignored by hash_set_asset_content
                sha256: Some(sha256),
            };
            hash_set_asset_content_raw(&mut hasher, &set_content_args, &encoded_content.data);
        }
    }

    let hash: [u8; 32] = hasher.finalize().into();
    Ok(hex::encode(hash))
}

fn default_encoders(media_type: &Mime) -> Vec<ContentEncoder> {
    match (media_type.type_(), media_type.subtype()) {
        (mime::TEXT, _) | (_, mime::JAVASCRIPT) | (_, mime::HTML) => {
            vec![ContentEncoder::Identity, ContentEncoder::Gzip]
        }
        _ => vec![ContentEncoder::Identity],
    }
}

fn hash_operation(
    hasher: &mut Sha256,
    op: &BatchOperationKind,
    project_assets: &HashMap<String, ProjectAsset>,
) -> Result<(), HashContentError> {
    match op {
        BatchOperationKind::CreateAsset(args) => hash_create_asset(hasher, args),
        BatchOperationKind::SetAssetContent(args) => {
            hash_set_asset_content(hasher, args, project_assets)?
        }
        BatchOperationKind::UnsetAssetContent(args) => hash_unset_asset_content(hasher, args),
        BatchOperationKind::DeleteAsset(args) => hash_delete_asset(hasher, args),
        BatchOperationKind::Clear(args) => hash_clear(hasher, args),
        BatchOperationKind::SetAssetProperties(args) => hash_set_asset_properties(hasher, args),
    };
    Ok(())
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
    hash_opt_bool(hasher, args.allow_raw_access);
    hash_opt_bool(hasher, args.enable_aliasing);
}

fn hash_set_asset_content(
    hasher: &mut Sha256,
    args: &SetAssetContentArguments,
    project_assets: &HashMap<String, ProjectAsset>,
) -> Result<(), HashContentError> {
    let project_asset = project_assets.get(&args.key).unwrap();
    let ad = &project_asset.asset_descriptor;

    let content = {
        let identity = Content::load(&ad.source)?;
        match args.content_encoding.as_str() {
            "identity" => identity,
            "br" | "brotli" => identity
                .encode(&Brotli)
                .map_err(|e| EncodeContentFailed(ad.key.clone(), Brotli, e))?,
            "gzip" => identity
                .encode(&Gzip)
                .map_err(|e| EncodeContentFailed(ad.key.clone(), Gzip, e))?,
            _ => unreachable!("unhandled content encoder"),
        }
    };

    hash_set_asset_content_raw(hasher, args, &content.data);
    Ok(())
}

fn hash_set_asset_content_raw(
    hasher: &mut Sha256,
    args: &SetAssetContentArguments,
    content_data: &[u8],
) {
    hasher.update(TAG_SET_ASSET_CONTENT);
    hash_str(hasher, &args.key);
    hash_str(hasher, &args.content_encoding);
    hash_opt_vec_u8(hasher, args.sha256.as_ref());
    // Length-prefixes the content, which makes the operation self-delimiting even though the
    // asset canister hashes the content bytes chunk by chunk.
    hash_len(hasher, content_data.len());

    // When hashing for state hash, we iterate over chunks.
    // Since content_data is the full content, updating with it is equivalent to updating with chunks sequentially.
    if content_data.len() > MAX_CHUNK_SIZE {
        for chunk in content_data.chunks(MAX_CHUNK_SIZE) {
            hasher.update(chunk);
        }
    } else {
        hasher.update(content_data);
    }
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

fn hash_opt_vec_u8(hasher: &mut Sha256, buf: Option<&Vec<u8>>) {
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
        for k in headers.keys() {
            let v = headers.get(k).unwrap();
            hash_str(hasher, k);
            hash_str(hasher, v);
        }
    } else {
        hasher.update(TAG_NONE);
    }
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
        if let Some(h) = headers {
            let h = BTreeMap::from_iter(h.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            hash_headers(hasher, Some(&h));
        } else {
            hash_headers(hasher, None);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed batch and the evidence the encoding above produces for it.
    ///
    /// `ic-certified-assets` has the same vector in
    /// `tests::evidence_computation::evidence_of_known_batch`.  The asset canister and this crate
    /// have to hash a batch to the same value -- comparing the two is the whole point of computing
    /// evidence here -- but the two implementations are separate, so each one pins the vector and
    /// a change to either encoding that is not made to the other shows up as a failure here.
    const KNOWN_BATCH_EVIDENCE: &str =
        "984350ced49ad34f6d48ddaddebd285f04c6cdeda6801053d0157f3b3c778ca6";

    #[test]
    fn evidence_of_known_batch() {
        const CONTENT: &[u8] = b"<!DOCTYPE html><html></html>";

        let mut hasher = Sha256::new();
        hasher.update(ENCODING_DOMAIN);

        hash_create_asset(
            &mut hasher,
            &CreateAssetArguments {
                key: "/index.html".to_string(),
                content_type: "text/html".to_string(),
                max_age: Some(600),
                headers: Some(BTreeMap::from([
                    ("X-Frame-Options".to_string(), "DENY".to_string()),
                    ("X-XSS-Protection".to_string(), "1; mode=block".to_string()),
                ])),
                enable_aliasing: Some(true),
                allow_raw_access: Some(false),
            },
        );

        let content_sha256: [u8; 32] = Sha256::digest(CONTENT).into();
        hash_set_asset_content_raw(
            &mut hasher,
            &SetAssetContentArguments {
                key: "/index.html".to_string(),
                content_encoding: "identity".to_string(),
                chunk_ids: vec![],
                last_chunk: None,
                sha256: Some(content_sha256.to_vec()),
            },
            CONTENT,
        );

        hash_delete_asset(
            &mut hasher,
            &DeleteAssetArguments {
                key: "/obsolete.txt".to_string(),
            },
        );

        let evidence: [u8; 32] = hasher.finalize().into();
        assert_eq!(hex::encode(evidence), KNOWN_BATCH_EVIDENCE);
    }
}
