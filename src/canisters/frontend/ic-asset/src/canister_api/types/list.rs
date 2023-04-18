use candid::CandidType;

/// Return a list of all assets in the canister.
#[derive(CandidType, Debug)]
pub struct ListAssetsRequest {}
