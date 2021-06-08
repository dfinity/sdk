use std::path::{Path, PathBuf};

pub fn canister_did_location(build_output_root: &Path, canister_name: &str) -> PathBuf {
    let part = format!("{}/{}.did", canister_name, canister_name);
    build_output_root.join(part)
}
