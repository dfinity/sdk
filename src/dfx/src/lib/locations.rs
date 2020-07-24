use std::path::PathBuf;

pub fn canister_did_location(build_output_root: &PathBuf, canister_name: &str) -> PathBuf {
    let part = format!("{}/{}.did", canister_name, canister_name);
    build_output_root.join(part)
}
