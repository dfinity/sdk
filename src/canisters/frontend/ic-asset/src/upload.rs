use crate::asset::config::AssetConfig;
use crate::batch_upload::operations::BATCH_UPLOAD_API_VERSION;
use crate::batch_upload::{
    self,
    operations::AssetDeletionReason,
    plumbing::{make_project_assets, AssetDescriptor, ChunkUploader},
};
use crate::canister_api::methods::{
    api_version::api_version,
    batch::{commit_batch, create_batch},
    list::list_assets,
};
use crate::canister_api::types::batch_upload::v0;

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

    let chunk_upload_target = ChunkUploader::new(canister.clone(), batch_id.clone());

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
        HashMap::new(),
        batch_id,
    );

    let canister_api_version = api_version(canister).await;
    info!(logger, "Committing batch.");
    let response = match canister_api_version {
        0 => {
            let commit_batch_args_v0 = v0::CommitBatchArguments::try_from(commit_batch_args)
                .map_err(|e| anyhow!("Failed to downgrade from v1::CommitBatchArguments to v0::CommitBatchArguments: {}. Please upgrade your asset canister, or use older tooling (dfx<=v-0.13.1 or icx-asset<=0.20.0)", e))?;
            commit_batch(canister, commit_batch_args_v0).await
        }
        BATCH_UPLOAD_API_VERSION.. => commit_batch(canister, commit_batch_args).await,
    };
    response.map_err(|e| anyhow!("Failed to upload project assets to frontend canister: {e}"))?;

    Ok(())
}
