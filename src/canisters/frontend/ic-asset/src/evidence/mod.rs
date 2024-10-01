use crate::asset::content::Content;
use crate::asset::content_encoder::ContentEncoder::{Brotli, Gzip};
use crate::batch_upload::operations::assemble_batch_operations;
use crate::batch_upload::operations::AssetDeletionReason::Obsolete;
use crate::batch_upload::plumbing::{make_project_assets, ProjectAsset};
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
use crate::error::HashContentError::{EncodeContentFailed, LoadContentFailed};
use crate::sync::gather_asset_descriptors;
use ic_utils::Canister;
use sha2::{Digest, Sha256};
use slog::{info, trace, Logger};
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

/// Compute the hash ("evidence") over the batch operations required to update the assets
pub async fn compute_evidence(
    canister: &Canister<'_>,
    dirs: &[&Path],
    logger: &Logger,
) -> Result<String, ComputeEvidenceError> {
    let asset_descriptors = gather_asset_descriptors(dirs, logger)?;

    let canister_assets = list_assets(canister)
        .await
        .map_err(ComputeEvidenceError::ListAssets)?;
    info!(
        logger,
        "Fetching properties for all assets in the canister."
    );
    let canister_asset_properties = get_assets_properties(canister, &canister_assets).await?;

    info!(
        logger,
        "Computing evidence for batch operations for assets in the project.",
    );
    let project_assets =
        make_project_assets(None, asset_descriptors, &canister_assets, logger).await?;

    let mut operations = assemble_batch_operations(
        &project_assets,
        canister_assets,
        Obsolete,
        canister_asset_properties,
    );
    operations.sort();
    trace!(logger, "{:#?}", operations);

    let mut sha = Sha256::new();
    for op in operations {
        hash_operation(&mut sha, &op, &project_assets)?;
    }
    let evidence: [u8; 32] = sha.finalize().into();

    Ok(hex::encode(evidence))
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
    hasher.update(TAG_SET_ASSET_CONTENT);
    hasher.update(&args.key);
    hasher.update(&args.content_encoding);
    hash_opt_vec_u8(hasher, args.sha256.as_ref());

    let project_asset = project_assets.get(&args.key).unwrap();
    let ad = &project_asset.asset_descriptor;

    let content = {
        let identity = Content::load(&ad.source).map_err(LoadContentFailed)?;
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
    hasher.update(&content.data);
    Ok(())
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
