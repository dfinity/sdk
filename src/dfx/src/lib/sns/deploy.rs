//! Code for creating an SNS.
use anyhow::bail;
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::error::DfxResult;
use crate::lib::sns::sns_cli::call_sns_cli;
use crate::Environment;

/// Creates an SNS.  This requires funds but no proposal.
#[context("Failed to deploy SNS with config: {}", path.display())]
pub fn deploy_sns(env: &dyn Environment, path: &Path) -> DfxResult<String> {
    let did_file = "candid/nns-sns-wasm.did";
    if !Path::new(did_file).exists() {
        bail!("Missing did file at '{did_file}'.  Please run 'dfx nns import' to get the file.");
    }

    let args = vec![
        OsString::from("deploy"),
        OsString::from("--init-config-file"),
        OsString::from(path),
        OsString::from("--candid"),
        OsString::from(did_file),
    ];
    call_sns_cli(env, &args).map(|stdout| format!("Deployed SNS: {}\n{}", path.display(), stdout))
}
