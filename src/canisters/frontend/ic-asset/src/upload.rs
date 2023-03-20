use crate::asset_canister::batch::{commit_batch, create_batch};
use crate::asset_canister::list::list_assets;
use crate::asset_canister::protocol::CommitBatchArguments;
use crate::asset_config::AssetConfig;
use crate::operations::{assemble_batch_operations, AssetDeletionReason};
use crate::plumbing::{make_project_assets, AssetDescriptor};
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

    let canister_assets = list_assets(canister).await?;

    info!(logger, "Starting batch.");

    let batch_id = create_batch(canister).await?;

    info!(logger, "Staging contents of new and changed assets:");

    let project_assets = make_project_assets(
        canister,
        &batch_id,
        asset_descriptors,
        &canister_assets,
        logger,
    )
    .await?;

    let operations = assemble_batch_operations(
        project_assets,
        canister_assets,
        AssetDeletionReason::Incompatible,
    );

    info!(logger, "Committing batch.");

    let args = CommitBatchArguments {
        batch_id,
        operations,
    };

    commit_batch(canister, args).await?;

    Ok(())
}
