use crate::error::GetAssetPropertiesError;
use crate::error::GetAssetPropertiesError::GetAssetPropertiesFailed;
use crate::{
    batch_upload::retryable::retryable,
    canister_api::{
        methods::method_names::GET_ASSET_PROPERTIES,
        types::asset::{AssetDetails, AssetProperties, GetAssetPropertiesArgument},
    },
};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoffBuilder;
use futures_intrusive::sync::SharedSemaphore;
use ic_agent::{agent::RejectResponse, AgentError};
use ic_utils::call::SyncCall;
use ic_utils::Canister;
use std::{collections::HashMap, time::Duration};

const MAX_CONCURRENT_REQUESTS: usize = 50;

pub(crate) async fn get_assets_properties(
    canister: &Canister<'_>,
    canister_assets: &HashMap<String, AssetDetails>,
) -> Result<HashMap<String, AssetProperties>, GetAssetPropertiesError> {
    let semaphore = SharedSemaphore::new(true, MAX_CONCURRENT_REQUESTS);

    let asset_ids = canister_assets.keys().cloned().collect::<Vec<_>>();
    let futs = asset_ids
        .iter()
        .map(|asset_id| async {
            let _releaser = semaphore.acquire(1).await;

            let mut retry_policy = ExponentialBackoffBuilder::new()
                .with_initial_interval(Duration::from_secs(1))
                .with_max_interval(Duration::from_secs(16))
                .with_multiplier(2.0)
                .with_max_elapsed_time(Some(Duration::from_secs(300)))
                .build();

            loop {
                let response = get_asset_properties(canister, asset_id).await;

                match response {
                    Ok(asset_properties) => break Ok(asset_properties),
                    Err(agent_err) if !retryable(&agent_err) => {
                        break Err(agent_err);
                    }
                    Err(agent_err) => match retry_policy.next_backoff() {
                        Some(duration) => tokio::time::sleep(duration).await,
                        None => break Err(agent_err),
                    },
                };
            }
        })
        .collect::<Vec<_>>();

    let results = futures::future::join_all(futs).await;

    let mut all_assets_properties = HashMap::new();
    for (index, result) in results.into_iter().enumerate() {
        match result {
            Ok(asset_properties) => {
                all_assets_properties.insert(asset_ids[index].to_string(), asset_properties);
            }
            // older canisters don't have get_assets_properties method
            // therefore we can break the loop
            Err(AgentError::UncertifiedReject(RejectResponse { reject_message, .. }))
                if reject_message
                    .contains(&format!("has no query method '{GET_ASSET_PROPERTIES}'"))
                    || reject_message.contains("query method does not exist") =>
            {
                break;
            }
            Err(e) => {
                return Err(GetAssetPropertiesFailed(asset_ids[index].clone(), e));
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
