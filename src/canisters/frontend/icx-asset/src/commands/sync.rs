use ic_utils::Canister;
use std::path::Path;

use crate::{support, SyncOpts};

pub(crate) async fn sync(canister: &Canister<'_>, o: &SyncOpts) -> support::Result {
    let dirs: Vec<&Path> = o.directory.iter().map(|d| d.as_path()).collect();
    ic_asset::sync(canister, &dirs).await?;
    Ok(())
}
