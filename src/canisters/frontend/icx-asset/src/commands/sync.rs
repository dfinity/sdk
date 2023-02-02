use ic_utils::Canister;
use std::path::Path;

use crate::{support, SyncOpts};

pub(crate) async fn sync(canister: &Canister<'_>, o: &SyncOpts) -> support::Result {
    let dirs: Vec<&Path> = o.directory.iter().map(|d| d.as_path()).collect();
    let _root = slog::Logger::root(
        slog::Discard,
        slog::o!("key1" => "value1", "key2" => "value2"),
    );

    ic_asset::sync(canister, &dirs, &_root).await?;
    Ok(())
}
