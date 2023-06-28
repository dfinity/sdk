//! Code for creating an SNS.
use anyhow::bail;
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::call_bundled::call_bundled;
use crate::lib::error::DfxResult;
use dfx_core::config::cache::Cache;

/// Creates an SNS.  This requires funds but no proposal.
#[context("Failed to deploy SNS with config: {}", path.display())]
pub fn deploy_sns(cache: &dyn Cache, path: &Path) -> DfxResult<String> {
    // Note: It MAY be possible to get the did file location using existing sdk methods.
    let did_file = "candid/nns-sns-wasm.did";
    if !Path::new(did_file).exists() {
        bail!("Missing did file at '{did_file}'.  Please run 'dfx nns import' to get the file.");
    }

    // Note: The --network flag is not available at the moment,
    //       so this always applies to local canister IDs.
    //       This will have to be expanded to cover deployments to
    //       mainnet quite soon.
    let canister_ids_file = ".dfx/local/canister_ids.json";

    let args = vec![
        OsString::from("deploy"),
        OsString::from("--init-config-file"),
        OsString::from(path),
        OsString::from("--candid"),
        OsString::from(did_file),
        OsString::from("--save-to"),
        OsString::from(canister_ids_file),
    ];
    call_bundled(cache, "sns", &args).map(|stdout| {
        format!(
            "Deployed SNS:\nSNS config: {}\nCanister ID file: {}\n\n{}",
            path.display(),
            canister_ids_file,
            stdout
        )
    })
}
