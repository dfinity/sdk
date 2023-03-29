use crate::canister_api::methods::method_names::API_VERSION;

use ic_utils::call::SyncCall;
use ic_utils::Canister;

pub(crate) async fn api_version(canister: &Canister<'_>) -> u16 {
    canister
        .query_(API_VERSION)
        .build()
        .call()
        .await
        .map_or(0, |v: (u16,)| v.0)
}
