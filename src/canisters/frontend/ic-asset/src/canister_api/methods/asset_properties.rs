use std::collections::HashMap;

use ic_utils::call::SyncCall;
use ic_utils::Canister;

use crate::canister_api::{
    methods::method_names::GET_ASSET_PROPERTIES,
    types::asset::{AssetDetails, AssetProperties, GetAssetProperties},
};

pub(crate) async fn get_assets_properties(
    canister: &Canister<'_>,
    canister_assets: &HashMap<String, AssetDetails>,
) -> anyhow::Result<HashMap<String, AssetProperties>> {
    let mut all_assets_properties = HashMap::new();
    for asset_id in canister_assets.keys() {
        let asset_properties = get_asset_properties(canister, asset_id).await?;
        all_assets_properties.insert(asset_id.clone(), asset_properties);
    }

    Ok(all_assets_properties)
}

pub(crate) async fn get_asset_properties(
    canister: &Canister<'_>,
    asset_id: &str,
) -> anyhow::Result<AssetProperties> {
    let (asset_properties,): (AssetProperties,) = canister
        .query_(GET_ASSET_PROPERTIES)
        .with_arg(GetAssetProperties {
            key: asset_id.to_string(),
        })
        .build()
        .call()
        .await?;
    Ok(asset_properties)
}
