use crate::asset::config::AssetConfig;
use crate::batch_upload::plumbing::ChunkUploadTarget;
use crate::batch_upload::{
    self,
    operations::AssetDeletionReason,
    plumbing::{make_project_assets, AssetDescriptor},
};
use crate::canister_api::methods::{
    api_version::api_version,
    batch::{commit_batch, create_batch},
    list::list_assets,
};

use anyhow::anyhow;
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

    let chunk_upload_target = ChunkUploadTarget {
        canister,
        batch_id: &batch_id,
    };

    let project_assets = make_project_assets(
        Some(&chunk_upload_target),
        asset_descriptors,
        &canister_assets,
        logger,
    )
    .await?;

    let commit_batch_args = batch_upload::operations::assemble_commit_batch_arguments(
        project_assets,
        canister_assets,
        AssetDeletionReason::Incompatible,
        batch_id,
    );

    let canister_api_version = api_version(canister).await;
    info!(logger, "Committing batch.");
    match canister_api_version {
        0.. => {
            // in the next PR:
            // if BATCH_UPLOAD_API_VERSION == 1 {
            //     let commit_batch_args = commit_batch_args.try_into::<v0::CommitBatchArguments>()?;
            //     warn!(logger, "The asset canister is running an old version of the API. It will not be able to set assets properties.");
            // }
            commit_batch(canister, commit_batch_args)
                .await
                .map_err(|e| anyhow!("Incompatible canister API version: {}", e))?;
        }
    }

    Ok(())
}
