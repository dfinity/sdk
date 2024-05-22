use crate::SyncOpts;
use ic_utils::Canister;
use slog::Logger;
use std::path::Path;

pub(crate) async fn sync(
    canister: &Canister<'_>,
    o: &SyncOpts,
    logger: &Logger,
) -> anyhow::Result<()> {
    let dirs: Vec<&Path> = o.directory.iter().map(|d| d.as_path()).collect();
    ic_asset::sync(
        canister,
        &dirs,
        ic_asset::ExistingAssetStrategy::Delete,
        logger,
    )
    .await?;
    Ok(())
}
