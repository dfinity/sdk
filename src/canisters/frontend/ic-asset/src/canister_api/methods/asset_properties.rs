use crate::canister_api::{
    methods::method_names::GET_ASSET_PROPERTIES,
    types::asset::{AssetDetails, AssetProperties, GetAssetPropertiesArgument},
};
use crate::error::GetAssetPropertiesError;
use crate::error::GetAssetPropertiesError::GetAssetPropertiesFailed;
use ic_agent::{
    agent::{RejectCode, RejectResponse},
    AgentError,
};
use ic_utils::call::SyncCall;
use ic_utils::Canister;
use std::collections::HashMap;

pub(crate) async fn get_assets_properties(
    canister: &Canister<'_>,
    canister_assets: &HashMap<String, AssetDetails>,
) -> Result<HashMap<String, AssetProperties>, GetAssetPropertiesError> {
    let mut all_assets_properties = HashMap::new();
    for asset_id in canister_assets.keys() {
        match get_asset_properties(canister, asset_id).await {
            Ok(asset_properties) => {
                all_assets_properties.insert(asset_id.to_string(), asset_properties);
            }
            // older canisters don't have get_assets_properties method
            // therefore we can break the loop
            Err(AgentError::ReplicaError(RejectResponse {
                reject_code,
                reject_message,
                ..
            })) if reject_code == RejectCode::DestinationInvalid
                && (reject_message
                    .contains(&format!("has no query method '{GET_ASSET_PROPERTIES}'"))
                    || reject_message.contains("query method does not exist")) =>
            {
                break;
            }
            Err(e) => {
                return Err(GetAssetPropertiesFailed(asset_id.clone(), e));
            }
        }
    }

    Ok(all_assets_properties)
}

pub(crate) async fn get_asset_properties(
    canister: &Canister<'_>,
    asset_id: &str,
) -> Result<AssetProperties, AgentError> {
    let (asset_properties,): (AssetProperties,) = canister
        .query(GET_ASSET_PROPERTIES)
        .with_arg(GetAssetPropertiesArgument(asset_id.to_string()))
        .build()
        .call()
        .await?;
    Ok(asset_properties)
}
