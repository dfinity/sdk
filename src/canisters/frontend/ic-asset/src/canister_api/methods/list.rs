use crate::canister_api::methods::method_names::LIST;
use crate::canister_api::types::{asset::AssetDetails, list::ListAssetsRequest};
use ic_utils::call::SyncCall;
use ic_utils::error::BaseError;
use ic_utils::Canister;
use std::collections::HashMap;

pub(crate) async fn list_assets(
    canister: &Canister<'_>,
) -> Result<HashMap<String, AssetDetails>, BaseError> {
    let (entries,): (Vec<AssetDetails>,) = canister
        .query(LIST)
        .with_arg(ListAssetsRequest {})
        .build()
        .call()
        .await?;

    let assets: HashMap<_, _> = entries.into_iter().map(|d| (d.key.clone(), d)).collect();

    Ok(assets)
}
