use crate::asset_canister::protocol::{
    AssetDetails, AssetProperties, BatchOperationKind, CreateAssetArguments, DeleteAssetArguments,
    SetAssetContentArguments, SetAssetPropertiesArguments, UnsetAssetContentArguments,
};
use crate::plumbing::ProjectAsset;
use std::collections::HashMap;

pub(crate) fn delete_obsolete_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_assets: &mut HashMap<String, AssetDetails>,
) {
    let mut deleted_canister_assets = vec![];
    for (key, canister_asset) in canister_assets.iter() {
        let project_asset = project_assets.get(key);
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
    for k in deleted_canister_assets {
        canister_assets.remove(&k);
    }
}

pub(crate) fn delete_incompatible_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_assets: &mut HashMap<String, AssetDetails>,
) {
    let mut deleted_canister_assets = vec![];
    for (key, canister_asset) in canister_assets.iter() {
        if let Some(project_asset) = project_assets.get(key) {
            if project_asset.media_type.to_string() != canister_asset.content_type {
                operations.push(BatchOperationKind::DeleteAsset(DeleteAssetArguments {
                    key: key.clone(),
                }));
                deleted_canister_assets.push(key.clone());
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
        for (content_encoding, v) in project_asset.encodings.iter() {
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
        if project_asset_properties.ne(&canister_asset_properties) {
            operations.push(BatchOperationKind::SetAssetProperties(
                // SetAssetPropertiesArguments {
                //     key: key.clone(),
                //     max_age: Some(
                //         project_asset_properties
                //             .cache
                //             .as_ref()
                //             .and_then(|c| c.max_age),
                //     ),
                //     headers: Some(project_asset_properties.headers),
                //     allow_raw_access: Some(project_asset_properties.allow_raw_access),
                // },
                SetAssetPropertiesArguments {
                    key: key.clone(),
                    max_age: None,
                    headers: None,
                    allow_raw_access: None,
                },
            ));
        }
    }
}
