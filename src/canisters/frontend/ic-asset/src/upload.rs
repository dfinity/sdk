use crate::asset::config::AssetConfig;
use crate::batch_upload;
use crate::batch_upload::operations::AssetDeletionReason;
use crate::batch_upload::plumbing::{make_project_assets, AssetDescriptor};
use crate::canister_api;
use crate::canister_api::methods::{
    api_version::api_version,
    batch::{commit_batch, create_batch},
    list::list_assets,
};

use anyhow::bail;
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

    match api_version(canister).await {
        0 => {
            let operations = batch_upload::operations::v0::assemble_batch_operations(
                project_assets,
                canister_assets,
                AssetDeletionReason::Incompatible,
            );
            info!(logger, "Committing batch.");
            let args = canister_api::types::batch_upload::v0::CommitBatchArguments {
                batch_id,
                operations,
            };
            commit_batch(canister, args).await?;
        }
        _ => bail!("unsupported API version"),
    }

    Ok(())
}
