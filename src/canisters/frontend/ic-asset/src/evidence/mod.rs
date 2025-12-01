use crate::AssetSyncProgressRenderer;
use crate::asset::content::Content;
use crate::asset::content_encoder::ContentEncoder::{self, Brotli, Gzip};
use crate::batch_upload::operations::AssetDeletionReason::Obsolete;
use crate::batch_upload::operations::assemble_batch_operations;
use crate::batch_upload::plumbing::{ProjectAsset, make_project_assets};
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

const MAX_CHUNK_SIZE: usize = 1_900_000;

/// Compute the hash ("evidence") over the batch operations required to update the assets
pub async fn compute_evidence(
    canister: &Canister<'_>,
    dirs: &[&Path],
    logger: &Logger,
    progress: Option<&dyn AssetSyncProgressRenderer>,
) -> Result<String, ComputeEvidenceError> {
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
    for op in operations {
        hash_operation(&mut sha, &op, &project_assets)?;
    }
    let evidence: [u8; 32] = sha.finalize().into();

    Ok(hex::encode(evidence))
}

/// Locally computes the state hash of the asset canister if it were synchronized with the given directories.
pub fn compute_state_hash(dirs: &[&Path], logger: &Logger) -> Result<[u8; 32], SyncError> {
    let asset_descriptors = gather_asset_descriptors(dirs, logger)
        .map_err(UploadContentError::GatherAssetDescriptorsFailed)
        .map_err(SyncError::UploadContentFailed)?;
    let mut sorted_asset_descriptors = asset_descriptors;
    sorted_asset_descriptors.sort_by(|a, b| a.key.cmp(&b.key));

    let mut hasher = Sha256::new();

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

    Ok(hasher.finalize().into())
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
    hasher.update(&args.key);
    hasher.update(&args.content_encoding);
    hash_opt_vec_u8(hasher, args.sha256.as_ref());

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
        hasher.update(buf);
    } else {
        hasher.update(TAG_NONE);
    }
}

fn hash_headers(hasher: &mut Sha256, headers: Option<&BTreeMap<String, String>>) {
    if let Some(headers) = headers {
        hasher.update(TAG_SOME);
        for k in headers.keys() {
            let v = headers.get(k).unwrap();
            hasher.update(k);
            hasher.update(v);
        }
    } else {
        hasher.update(TAG_NONE);
    }
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
mod test_compute_state_hash {
    use super::*;
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::Builder;

    fn create_temporary_assets_directory(files: HashMap<String, String>) -> tempfile::TempDir {
        let assets_dir = Builder::new().prefix("assets").tempdir().unwrap();
        for (name, content) in files {
            let path = assets_dir.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            let mut file = File::create(path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }
        assets_dir
    }

    #[test]
    fn compute_hash_stability() {
        let files = HashMap::from([
            ("asset1.txt".to_string(), "content1".to_string()),
            ("subdir/asset2.txt".to_string(), "content2".to_string()),
        ]);
        let temp_dir = create_temporary_assets_directory(files);
        let logger = slog::Logger::root(slog::Discard, slog::o!());

        let hash1 = compute_state_hash(&[temp_dir.path()], &logger).unwrap();
        let hash2 = compute_state_hash(&[temp_dir.path()], &logger).unwrap();
        assert_eq!(hash1, hash2);
    }
}
