use candid::Nat;

use crate::batch_upload::plumbing::ProjectAsset;
use crate::canister_api::types::asset::AssetDetails;
use crate::canister_api::types::batch_upload::common::{
    CreateAssetArguments, DeleteAssetArguments, SetAssetContentArguments,
    UnsetAssetContentArguments,
};
use crate::canister_api::types::batch_upload::v0::{BatchOperationKind, CommitBatchArguments};
use std::collections::HashMap;

#[allow(dead_code)]
pub(crate) const BATCH_UPLOAD_API_VERSION: u16 = 0;

pub(crate) fn assemble_batch_operations(
    project_assets: HashMap<String, ProjectAsset>,
    canister_assets: HashMap<String, AssetDetails>,
    asset_deletion_reason: AssetDeletionReason,
) -> Vec<BatchOperationKind> {
    let mut canister_assets = canister_assets;

    let mut operations = vec![];

    delete_assets(
        &mut operations,
        &project_assets,
        &mut canister_assets,
        asset_deletion_reason,
    );
    create_new_assets(&mut operations, &project_assets, &canister_assets);
    unset_obsolete_encodings(&mut operations, &project_assets, &canister_assets);
    set_encodings(&mut operations, project_assets);

    operations
}

pub(crate) fn assemble_commit_batch_arguments(
    project_assets: HashMap<String, ProjectAsset>,
    canister_assets: HashMap<String, AssetDetails>,
    asset_deletion_reason: AssetDeletionReason,
    batch_id: Nat,
) -> CommitBatchArguments {
    let operations =
        assemble_batch_operations(project_assets, canister_assets, asset_deletion_reason);
    CommitBatchArguments {
        operations,
        batch_id,
    }
}

pub(crate) enum AssetDeletionReason {
    Obsolete,
    Incompatible,
}

pub(crate) fn delete_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_assets: &mut HashMap<String, AssetDetails>,
    reason: AssetDeletionReason,
) {
    let mut deleted_canister_assets = vec![];
    for (key, canister_asset) in canister_assets.iter() {
        let project_asset = project_assets.get(key);
        match reason {
            AssetDeletionReason::Obsolete => {
                if project_asset
                    .filter(|&x| x.media_type.to_string() == canister_asset.content_type)
                    .is_none()
                {
                    operations.push(BatchOperationKind::DeleteAsset(DeleteAssetArguments {
                        key: key.clone(),
                    }));
                    deleted_canister_assets.push(key.clone());
                }
            }
            AssetDeletionReason::Incompatible => {
                if let Some(project_asset) = project_assets.get(key) {
                    if project_asset.media_type.to_string() != canister_asset.content_type {
                        operations.push(BatchOperationKind::DeleteAsset(DeleteAssetArguments {
                            key: key.clone(),
                        }));
                        deleted_canister_assets.push(key.clone());
                    }
                }
            }
        }
    }
    for k in deleted_canister_assets {
        canister_assets.remove(&k);
    }
}

pub(crate) fn create_new_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_assets: &HashMap<String, AssetDetails>,
) {
    for (key, project_asset) in project_assets {
        if !canister_assets.contains_key(key) {
            let max_age = project_asset
                .asset_descriptor
                .config
                .cache
                .as_ref()
                .and_then(|c| c.max_age);

            let headers = project_asset.asset_descriptor.config.clone().headers;
            let enable_aliasing = project_asset.asset_descriptor.config.enable_aliasing;
            let allow_raw_access = project_asset.asset_descriptor.config.allow_raw_access;

            operations.push(BatchOperationKind::CreateAsset(CreateAssetArguments {
                key: key.clone(),
                content_type: project_asset.media_type.to_string(),
                max_age,
                headers,
                enable_aliasing,
                allow_raw_access,
            }));
        }
    }
}

pub(crate) fn unset_obsolete_encodings(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_assets: &HashMap<String, AssetDetails>,
) {
    for (key, details) in canister_assets {
        // delete_obsolete_assets handles the case where key is not found in project_assets
        if let Some(project_asset) = project_assets.get(key) {
            for encoding_details in &details.encodings {
                let project_contains_encoding = project_asset
                    .encodings
                    .contains_key(&encoding_details.content_encoding);
                if !project_contains_encoding {
                    operations.push(BatchOperationKind::UnsetAssetContent(
                        UnsetAssetContentArguments {
                            key: key.clone(),
                            content_encoding: encoding_details.content_encoding.clone(),
                        },
                    ));
                }
            }
        }
    }
}

pub(crate) fn set_encodings(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: HashMap<String, ProjectAsset>,
) {
    for (key, project_asset) in project_assets {
        for (content_encoding, v) in project_asset.encodings {
            if v.already_in_place {
                continue;
            }

            operations.push(BatchOperationKind::SetAssetContent(
                SetAssetContentArguments {
                    key: key.clone(),
                    content_encoding,
                    chunk_ids: v.chunk_ids,
                    sha256: Some(v.sha256),
                },
            ));
        }
    }
}
