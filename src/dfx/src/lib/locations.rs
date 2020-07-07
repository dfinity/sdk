use std::path::PathBuf;

pub fn canister_did_location(canister_name: &str ) -> PathBuf {
    PathBuf::from(format!("canisters/{}/{}.did", canister_name, canister_name))
}
