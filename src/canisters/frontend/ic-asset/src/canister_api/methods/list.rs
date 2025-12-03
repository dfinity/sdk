use crate::canister_api::methods::method_names::LIST;
use crate::canister_api::types::{asset::AssetDetails, list::ListAssetsRequest};
use candid::Nat;
use ic_agent::AgentError;
use ic_utils::Canister;
use ic_utils::call::SyncCall;
use std::collections::HashMap;

/// Lists all assets in the canister. Handles pagination transparently.
pub async fn list_assets(
    canister: &Canister<'_>,
) -> Result<HashMap<String, AssetDetails>, AgentError> {
    let mut all_entries: Vec<AssetDetails> = Vec::new();
    let mut start = 0u64;
    let mut prev_page_size: Option<usize> = None;

    // Fetch assets in pages until we get 0 items or fewer items than the previous page
    loop {
        let (entries,): (Vec<AssetDetails>,) = canister
            .query(LIST)
            .with_arg(ListAssetsRequest {
                start: Some(Nat::from(start)),
                length: None,
            })
            .build()
            .call()
            .await?;

        let num_entries = entries.len();
        if num_entries == 0 {
            break;
        }

        // If we're on a subsequent page but got the same data as the first page,
        // the canister doesn't support pagination and is returning all entries every time
        if start > 0 && entries == all_entries {
            break;
        }

        start += num_entries as u64;
        all_entries.extend(entries);

        // If we got fewer items than the previous page, we've reached the end
        if let Some(prev_size) = prev_page_size {
            if num_entries < prev_size {
                break;
            }
        }
        prev_page_size = Some(num_entries);
    }

    let assets: HashMap<_, _> = all_entries
        .into_iter()
        .map(|d| (d.key.clone(), d))
        .collect();

    Ok(assets)
}
