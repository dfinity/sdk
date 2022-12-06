use crate::asset_canister::method_names::LIST;
use crate::asset_canister::protocol::{AssetDetails, ListAssetsRequest};
use ic_utils::call::SyncCall;
use ic_utils::Canister;

use std::collections::HashMap;

pub(crate) async fn list_assets(
    canister: &Canister<'_>,
) -> anyhow::Result<HashMap<String, AssetDetails>> {
    let (entries,): (Vec<AssetDetails>,) = canister
        .query_(LIST)
        .with_arg(ListAssetsRequest {})
        .build()
        .call()
        .await?;

    let assets: HashMap<_, _> = entries.into_iter().map(|d| (d.key.clone(), d)).collect();

    Ok(assets)
}
