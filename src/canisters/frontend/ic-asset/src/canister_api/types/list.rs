use candid::{CandidType, Nat};

/// Return a list of all assets in the canister.
#[derive(CandidType, Debug)]
pub struct ListAssetsRequest {
    pub start: Option<Nat>,
    pub length: Option<Nat>,
}
