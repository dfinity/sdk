use crate::asset_canister::protocol::{
    AssetDetails, BatchOperationKind, CreateAssetArguments, DeleteAssetArguments,
    SetAssetContentArguments, UnsetAssetContentArguments,
};
use crate::plumbing::ProjectAsset;
use std::collections::HashMap;

pub(crate) fn delete_obsolete_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    container_assets: &mut HashMap<String, AssetDetails>,
) {
    let mut deleted_container_assets = vec![];
    for (key, container_asset) in container_assets.iter() {
        let project_asset = project_assets.get(key);
        if project_asset
            .filter(|&x| x.media_type.to_string() == container_asset.content_type)
            .is_none()
        {
            operations.push(BatchOperationKind::DeleteAsset(DeleteAssetArguments {
                key: key.clone(),
            }));
            deleted_container_assets.push(key.clone());
        }
    }
    for k in deleted_container_assets {
        container_assets.remove(&k);
    }
}

pub(crate) fn delete_incompatible_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    container_assets: &mut HashMap<String, AssetDetails>,
) {
    let mut deleted_container_assets = vec![];
    for (key, container_asset) in container_assets.iter() {
        if let Some(project_asset) = project_assets.get(key) {
            if project_asset.media_type.to_string() != container_asset.content_type {
                operations.push(BatchOperationKind::DeleteAsset(DeleteAssetArguments {
                    key: key.clone(),
                }));
                deleted_container_assets.push(key.clone());
            }
        }
    }
    for k in deleted_container_assets {
        container_assets.remove(&k);
    }
}

pub(crate) fn create_new_assets(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    container_assets: &HashMap<String, AssetDetails>,
) {
    for (key, project_asset) in project_assets {
        if !container_assets.contains_key(key) {
            let max_age = project_asset
                .asset_descriptor
                .config
                .cache
                .as_ref()
                .and_then(|c| c.max_age);

            let headers = project_asset.asset_descriptor.config.clone().headers;

            operations.push(BatchOperationKind::CreateAsset(CreateAssetArguments {
                key: key.clone(),
                content_type: project_asset.media_type.to_string(),
                max_age,
                headers,
            }));
        }
    }
}

pub(crate) fn unset_obsolete_encodings(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    container_assets: &HashMap<String, AssetDetails>,
) {
    for (key, details) in container_assets {
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
