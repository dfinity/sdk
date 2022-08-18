use ic_utils::Canister;
use std::path::Path;

use crate::{support, SyncOpts};
use std::time::Duration;

pub(crate) async fn sync(
    canister: &Canister<'_>,
    timeout: Duration,
    o: &SyncOpts,
) -> support::Result {
    let dirs: Vec<&Path> = o.directory.iter().map(|d| d.as_path()).collect();
    ic_asset::sync(canister, &dirs, timeout).await?;
    Ok(())
}
