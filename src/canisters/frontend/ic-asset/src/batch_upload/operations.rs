use candid::Nat;

use crate::batch_upload::plumbing::ProjectAsset;
use crate::canister_api::types::asset::{
    AssetDetails, AssetProperties, SetAssetPropertiesArguments,
};
use crate::canister_api::types::batch_upload::common::{
    CreateAssetArguments, DeleteAssetArguments, SetAssetContentArguments,
    UnsetAssetContentArguments,
};
use crate::canister_api::types::batch_upload::v1::{BatchOperationKind, CommitBatchArguments};
use std::collections::{BTreeMap, HashMap};

pub(crate) const BATCH_UPLOAD_API_VERSION: u16 = 1;

pub(crate) fn assemble_batch_operations(
    project_assets: &HashMap<String, ProjectAsset>,
    canister_assets: HashMap<String, AssetDetails>,
    asset_deletion_reason: AssetDeletionReason,
    canister_asset_properties: HashMap<String, AssetProperties>,
) -> Vec<BatchOperationKind> {
    let mut canister_assets = canister_assets;

    let mut operations = vec![];

    delete_assets(
        &mut operations,
        project_assets,
        &mut canister_assets,
        asset_deletion_reason,
    );
    create_new_assets(&mut operations, project_assets, &canister_assets);
    unset_obsolete_encodings(&mut operations, project_assets, &canister_assets);
    set_encodings(&mut operations, project_assets);
    update_properties(&mut operations, project_assets, &canister_asset_properties);

    operations
}

pub(crate) fn assemble_commit_batch_arguments(
    project_assets: HashMap<String, ProjectAsset>,
    canister_assets: HashMap<String, AssetDetails>,
    asset_deletion_reason: AssetDeletionReason,
    canister_asset_properties: HashMap<String, AssetProperties>,
    batch_id: Nat,
) -> CommitBatchArguments {
    let operations = assemble_batch_operations(
        &project_assets,
        canister_assets,
        asset_deletion_reason,
        canister_asset_properties,
    );
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
    project_assets: &HashMap<String, ProjectAsset>,
) {
    for (key, project_asset) in project_assets {
        for (content_encoding, v) in &project_asset.encodings {
            if v.already_in_place {
                continue;
            }

            operations.push(BatchOperationKind::SetAssetContent(
                SetAssetContentArguments {
                    key: key.clone(),
                    content_encoding: content_encoding.clone(),
                    chunk_ids: v.chunk_ids.clone(),
                    sha256: Some(v.sha256.clone()),
                },
            ));
        }
    }
}

pub(crate) fn update_properties(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_asset_properties: &HashMap<String, AssetProperties>,
) {
    for (key, project_asset) in project_assets {
        let project_asset_properties = project_asset.asset_descriptor.config.clone();
        let canister_asset_properties = canister_asset_properties.get(key);
        // skip if the asset is not already in the canister, because
        // proporties gonna be created during create_new_assets call
        if canister_asset_properties.is_none() {
            continue;
        }
        let canister_asset_properties = canister_asset_properties.unwrap();
        let cache_is_different = project_asset_properties
            .cache
            .as_ref()
            .and_then(|v| v.max_age)
            != canister_asset_properties.max_age;
        let headers_are_different = project_asset_properties.headers
            != canister_asset_properties
                .headers
                .as_ref()
                .map(|v| BTreeMap::from_iter(v.clone().into_iter()));
        let allow_raw_access_is_different =
            project_asset_properties.allow_raw_access != canister_asset_properties.allow_raw_access;

        // check if the properties are the same and skip if they are to save saves cycles
        if cache_is_different || headers_are_different || allow_raw_access_is_different {
            operations.push(BatchOperationKind::SetAssetProperties(
                SetAssetPropertiesArguments {
                    key: key.clone(),
                    max_age: Some(
                        project_asset_properties
                            .cache
                            .as_ref()
                            .and_then(|c| c.max_age),
                    ),
                    headers: Some(project_asset_properties.headers.map(|hm| {
                        hm.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect::<Vec<_>>()
                    })),
                    is_aliased: Some(project_asset_properties.enable_aliasing),
                    allow_raw_access: Some(project_asset_properties.allow_raw_access),
                },
            ));
        }
    }
}
