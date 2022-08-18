use crate::asset_canister::batch::{commit_batch, create_batch};
use crate::asset_canister::list::list_assets;
use crate::asset_canister::protocol::{AssetDetails, BatchOperationKind};
use crate::asset_config::AssetConfig;
use crate::operations::{
    create_new_assets, delete_incompatible_assets, set_encodings, unset_obsolete_encodings,
};
use crate::params::CanisterCallParams;
use crate::plumbing::{make_project_assets, AssetDescriptor, ProjectAsset};
use ic_utils::Canister;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Upload the specified files
pub async fn upload(
    canister: &Canister<'_>,
    timeout: Duration,
    files: HashMap<String, PathBuf>,
) -> anyhow::Result<()> {
    let asset_descriptors: Vec<AssetDescriptor> = files
        .iter()
        .map(|x| AssetDescriptor {
            source: x.1.clone(),
            key: x.0.clone(),
            config: AssetConfig::default(),
        })
        .collect();

    let canister_call_params = CanisterCallParams { canister, timeout };

    let container_assets = list_assets(&canister_call_params).await?;

    println!("Starting batch.");

    let batch_id = create_batch(&canister_call_params).await?;

    println!("Staging contents of new and changed assets:");

    let project_assets = make_project_assets(
        &canister_call_params,
        &batch_id,
        asset_descriptors,
        &container_assets,
    )
    .await?;

    let operations = assemble_upload_operations(project_assets, container_assets);

    println!("Committing batch.");

    commit_batch(&canister_call_params, &batch_id, operations).await?;

    Ok(())
}

fn assemble_upload_operations(
    project_assets: HashMap<String, ProjectAsset>,
    container_assets: HashMap<String, AssetDetails>,
) -> Vec<BatchOperationKind> {
    let mut container_assets = container_assets;

    let mut operations = vec![];

    delete_incompatible_assets(&mut operations, &project_assets, &mut container_assets);
    create_new_assets(&mut operations, &project_assets, &container_assets);
    unset_obsolete_encodings(&mut operations, &project_assets, &container_assets);
    set_encodings(&mut operations, project_assets);

    operations
}
