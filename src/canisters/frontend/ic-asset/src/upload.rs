use crate::asset_canister::batch::{commit_batch, create_batch};
use crate::asset_canister::list::list_assets;
use crate::asset_canister::protocol::{AssetDetails, BatchOperationKind, CommitBatchArguments};
use crate::asset_config::AssetConfig;
use crate::operations::{
    create_new_assets, delete_incompatible_assets, set_encodings, unset_obsolete_encodings,
};
use crate::plumbing::{make_project_assets, AssetDescriptor, ProjectAsset};
use ic_utils::Canister;
use slog::{info, Logger};
use std::collections::HashMap;
use std::path::PathBuf;

/// Upload the specified files
pub async fn upload(
    canister: &Canister<'_>,
    files: HashMap<String, PathBuf>,
    logger: &Logger,
) -> anyhow::Result<()> {
    let asset_descriptors: Vec<AssetDescriptor> = files
        .iter()
        .map(|x| AssetDescriptor {
            source: x.1.clone(),
            key: x.0.clone(),
            config: AssetConfig::default(),
        })
        .collect();

    let container_assets = list_assets(canister).await?;

    info!(logger, "Starting batch.");

    let batch_id = create_batch(canister).await?;

    info!(logger, "Staging contents of new and changed assets:");

    let project_assets = make_project_assets(
        canister,
        &batch_id,
        asset_descriptors,
        &container_assets,
        logger,
    )
    .await?;

    let operations = assemble_upload_operations(project_assets, container_assets);

    info!(logger, "Committing batch.");

    let args = CommitBatchArguments {
        batch_id,
        operations,
    };

    commit_batch(canister, args).await?;

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
