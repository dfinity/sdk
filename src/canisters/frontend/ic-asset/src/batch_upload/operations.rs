use crate::batch_upload::plumbing::ProjectAsset;
use crate::canister_api::types::asset::{
    AssetDetails, AssetProperties, SetAssetPropertiesArguments,
};
use crate::canister_api::types::batch_upload::common::{
    CreateAssetArguments, DeleteAssetArguments, SetAssetContentArguments,
    UnsetAssetContentArguments,
};
use crate::canister_api::types::batch_upload::v1::{BatchOperationKind, CommitBatchArguments};
use crate::error::{AssembleCommitBatchArgumentError, SetEncodingError};
use candid::Nat;
use std::collections::HashMap;

use super::plumbing::ChunkUploader;

pub(crate) const BATCH_UPLOAD_API_VERSION: u16 = 1;

pub(crate) async fn assemble_batch_operations(
    chunk_uploader: Option<&ChunkUploader<'_>>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_assets: HashMap<String, AssetDetails>,
    asset_deletion_reason: AssetDeletionReason,
    canister_asset_properties: HashMap<String, AssetProperties>,
    insecure_dev_mode: bool,
) -> Result<Vec<BatchOperationKind>, AssembleCommitBatchArgumentError> {
    let mut canister_assets = canister_assets;

    let mut operations = vec![];

    delete_assets(
        &mut operations,
        project_assets,
        &mut canister_assets,
        asset_deletion_reason,
    );
    create_new_assets(
        &mut operations,
        project_assets,
        &canister_assets,
        insecure_dev_mode,
    );
    unset_obsolete_encodings(&mut operations, project_assets, &canister_assets);
    set_encodings(&mut operations, chunk_uploader, project_assets)
        .await
        .map_err(AssembleCommitBatchArgumentError::SetEncodingFailed)?;
    update_properties(
        &mut operations,
        project_assets,
        &canister_asset_properties,
        insecure_dev_mode,
    );

    Ok(operations)
}

pub(crate) async fn assemble_commit_batch_arguments(
    chunk_uploader: &ChunkUploader<'_>,
    project_assets: HashMap<String, ProjectAsset>,
    canister_assets: HashMap<String, AssetDetails>,
    asset_deletion_reason: AssetDeletionReason,
    canister_asset_properties: HashMap<String, AssetProperties>,
    batch_id: Nat,
    insecure_dev_mode: bool,
) -> Result<CommitBatchArguments, AssembleCommitBatchArgumentError> {
    let operations = assemble_batch_operations(
        Some(chunk_uploader),
        &project_assets,
        canister_assets,
        asset_deletion_reason,
        canister_asset_properties,
        insecure_dev_mode,
    )
    .await?;
    Ok(CommitBatchArguments {
        operations,
        batch_id,
    })
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
    insecure_dev_mode: bool,
) {
    for (key, project_asset) in project_assets {
        if !canister_assets.contains_key(key) {
            let max_age = project_asset
                .asset_descriptor
                .config
                .cache
                .as_ref()
                .and_then(|c| c.max_age);

            let headers = project_asset
                .asset_descriptor
                .config
                .combined_headers(insecure_dev_mode);
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

pub(crate) async fn set_encodings(
    operations: &mut Vec<BatchOperationKind>,
    chunk_uploader: Option<&ChunkUploader<'_>>,
    project_assets: &HashMap<String, ProjectAsset>,
) -> Result<(), SetEncodingError> {
    for (key, project_asset) in project_assets {
        for (content_encoding, v) in &project_asset.encodings {
            if v.already_in_place {
                continue;
            }
            let (chunk_ids, last_chunk) = match chunk_uploader {
                Some(uploader) => {
                    uploader
                        .uploader_ids_to_canister_chunk_ids(&v.uploader_chunk_ids)
                        .await?
                }
                None => (vec![], None),
            };
            operations.push(BatchOperationKind::SetAssetContent(
                SetAssetContentArguments {
                    key: key.clone(),
                    content_encoding: content_encoding.clone(),
                    chunk_ids,
                    last_chunk,
                    sha256: Some(v.sha256.clone()),
                },
            ));
        }
    }
    Ok(())
}

pub(crate) fn update_properties(
    operations: &mut Vec<BatchOperationKind>,
    project_assets: &HashMap<String, ProjectAsset>,
    canister_asset_properties: &HashMap<String, AssetProperties>,
    insecure_dev_mode: bool,
) {
    for (key, project_asset) in project_assets {
        let project_asset_properties = project_asset.asset_descriptor.config.clone();
        // skip if the asset is not already in the canister, because
        // properties are going to be created during create_new_assets call
        if let Some(canister_asset_properties) = canister_asset_properties.get(key) {
            let set_asset_props = SetAssetPropertiesArguments {
                key: key.clone(),
                max_age: {
                    let project_asset_max_age = project_asset_properties
                        .cache
                        .as_ref()
                        .and_then(|v| v.max_age);
                    if project_asset_max_age != canister_asset_properties.max_age {
                        Some(project_asset_max_age)
                    } else {
                        None
                    }
                },
                headers: {
                    let project_asset_headers = project_asset_properties
                        .combined_headers(insecure_dev_mode)
                        .map(|hm| {
                            let mut vec = Vec::from_iter(hm.into_iter());
                            vec.sort();
                            vec
                        });
                    let canister_asset_headers =
                        canister_asset_properties.headers.as_ref().map(|hm| {
                            // collect into a vec and sort it
                            let mut vec = Vec::from_iter(hm.clone().into_iter());
                            vec.sort();
                            vec
                        });
                    if project_asset_headers != canister_asset_headers {
                        Some(project_asset_headers)
                    } else {
                        None
                    }
                },
                is_aliased: {
                    if project_asset_properties.enable_aliasing
                        != canister_asset_properties.is_aliased
                    {
                        Some(project_asset_properties.enable_aliasing)
                    } else {
                        None
                    }
                },
                allow_raw_access: {
                    if project_asset_properties.allow_raw_access
                        != canister_asset_properties.allow_raw_access
                    {
                        Some(project_asset_properties.allow_raw_access)
                    } else {
                        None
                    }
                },
            };
            // check if the properties are the same and skip if they are to save saves cycles
            if set_asset_props.allow_raw_access.is_some()
                || set_asset_props.max_age.is_some()
                || set_asset_props.headers.is_some()
                || set_asset_props.is_aliased.is_some()
            {
                operations.push(BatchOperationKind::SetAssetProperties(set_asset_props));
            }
        }
    }
}

#[cfg(test)]
mod test_update_properties {
    use super::update_properties;
    use crate::asset::config::{AssetConfig, CacheConfig};
    use crate::batch_upload::plumbing::{AssetDescriptor, ProjectAsset};
    use crate::canister_api::types::asset::{AssetProperties, SetAssetPropertiesArguments};
    use crate::canister_api::types::batch_upload::v1::BatchOperationKind;
    use std::collections::{BTreeMap, HashMap};
    use std::path::PathBuf;

    fn dummy_project_asset(key: &str, asset_props: AssetConfig) -> ProjectAsset {
        ProjectAsset {
            media_type: mime::TEXT_PLAIN,
            encodings: HashMap::new(),
            asset_descriptor: AssetDescriptor {
                key: key.to_string(),
                source: PathBuf::from(""),
                config: asset_props,
            },
        }
    }

    #[test]
    fn basic_test() {
        let mut project_assets = HashMap::new();
        let mut canister_asset_properties = HashMap::new();
        project_assets.insert(
            "key1".to_string(),
            dummy_project_asset(
                "key1",
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(100) }),
                    headers: Some(BTreeMap::from([("key".to_string(), "value".to_string())])),
                    enable_aliasing: Some(false),
                    allow_raw_access: Some(false),
                    ..Default::default()
                },
            ),
        );
        project_assets.insert(
            "key2".to_string(),
            dummy_project_asset(
                "key2",
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(100) }),
                    headers: Some(BTreeMap::new()),
                    enable_aliasing: Some(true),
                    allow_raw_access: Some(true),
                    ..Default::default()
                },
            ),
        );
        canister_asset_properties.insert(
            "key1".to_string(),
            AssetProperties {
                max_age: Some(1),
                headers: Some(HashMap::new()),
                is_aliased: Some(true),
                allow_raw_access: Some(true),
            },
        );
        let mut operations = vec![];
        update_properties(
            &mut operations,
            &project_assets,
            &canister_asset_properties,
            false,
        );
        assert_eq!(operations.len(), 1);
        assert_eq!(
            operations[0],
            BatchOperationKind::SetAssetProperties(SetAssetPropertiesArguments {
                key: "key1".to_string(),
                max_age: Some(Some(100)),
                headers: Some(Some(vec![("key".to_string(), "value".to_string())])),
                is_aliased: Some(Some(false)),
                allow_raw_access: Some(Some(false)),
            })
        );
    }

    #[test]
    fn update_no_properties() {
        let mut project_assets = HashMap::new();
        let mut canister_asset_properties = HashMap::new();
        project_assets.insert(
            "key1".to_string(),
            dummy_project_asset(
                "key1",
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(100) }),
                    headers: Some(BTreeMap::new()),
                    enable_aliasing: Some(true),
                    allow_raw_access: Some(true),
                    ..Default::default()
                },
            ),
        );
        project_assets.insert(
            "key2".to_string(),
            dummy_project_asset(
                "key2",
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(100) }),
                    headers: Some(BTreeMap::new()),
                    enable_aliasing: Some(true),
                    allow_raw_access: Some(true),
                    ..Default::default()
                },
            ),
        );
        canister_asset_properties.insert(
            "key1".to_string(),
            AssetProperties {
                max_age: Some(100),
                headers: Some(HashMap::new()),
                is_aliased: Some(true),
                allow_raw_access: Some(true),
            },
        );
        canister_asset_properties.insert(
            "key3".to_string(),
            AssetProperties {
                max_age: Some(100),
                headers: Some(HashMap::new()),
                is_aliased: Some(true),
                allow_raw_access: Some(true),
            },
        );
        let mut operations = vec![];
        update_properties(
            &mut operations,
            &project_assets,
            &canister_asset_properties,
            false,
        );
        assert_eq!(operations.len(), 0);
    }

    #[test]
    fn update_with_nones() {
        let mut project_assets = HashMap::new();
        let mut canister_asset_properties = HashMap::new();
        project_assets.insert(
            "key1".to_string(),
            dummy_project_asset(
                "key1",
                AssetConfig {
                    cache: None,
                    headers: None,
                    enable_aliasing: None,
                    allow_raw_access: None,
                    ..Default::default()
                },
            ),
        );
        canister_asset_properties.insert(
            "key1".to_string(),
            AssetProperties {
                max_age: Some(100),
                headers: Some(HashMap::from([("key".to_string(), "value".to_string())])),
                is_aliased: Some(true),
                allow_raw_access: Some(true),
            },
        );
        let mut operations = vec![];
        update_properties(
            &mut operations,
            &project_assets,
            &canister_asset_properties,
            false,
        );
        assert_eq!(operations.len(), 1);
        assert_eq!(
            operations[0],
            BatchOperationKind::SetAssetProperties(SetAssetPropertiesArguments {
                key: "key1".to_string(),
                max_age: Some(None),
                headers: Some(None),
                is_aliased: Some(None),
                allow_raw_access: Some(None),
            })
        );
    }
}
