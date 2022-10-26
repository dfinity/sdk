//! Code for creating an SNS.
use anyhow::{anyhow, bail, Context};
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::call_bundled::call_bundled;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::provider::create_agent_environment;
use crate::Environment;

/// Creates an SNS.  This requires funds but no proposal.
#[context("Failed to deploy SNS with config: {}", path.display())]
pub fn deploy_sns(
    env: &dyn Environment,
    path: &Path,
    network: Option<String>,
) -> DfxResult<String> {
    // For networks other than ic and local we need a provider URL:
    let agent_environment = create_agent_environment(env, network.clone())?;
    let network_descriptor = agent_environment.get_network_descriptor();
    // The underlying sns binary recognises "local", "ic" and provider URLs, so we need one of those:
    let sns_network_param = match network.as_ref().map(|network| network.as_ref()) {
        None => "local",
        Some("ic") => "ic",
        Some("local") => "local",
        Some(url) if url::Url::parse(url).is_ok() => url,
        _ => network_descriptor.first_provider().with_context(|| {
            format!(
                "Network '{}' has no known provider URL.",
                network.unwrap_or_default()
            )
        })?,
    };

    // Note: It MAY be possible to get the did file location using existing sdk methods.
    let did_file = "candid/nns-sns-wasm.did";
    if !Path::new(did_file).exists() {
        bail!("Missing did file at '{did_file}'.  Please run 'dfx nns import' to get the file.");
    }

    let canister_id_store = CanisterIdStore::new(network_descriptor, env.get_config())?;
    let canister_ids_file = canister_id_store
        .get_path()
        .ok_or_else(|| anyhow!("Unable to determine canister_ids file."))?;

    let args = vec![
        OsString::from("deploy"),
        OsString::from("--network"),
        OsString::from(sns_network_param),
        OsString::from("--init-config-file"),
        OsString::from(path),
        OsString::from("--candid"),
        OsString::from(did_file),
        OsString::from("--save-to"),
        OsString::from(canister_ids_file),
    ];
    call_bundled(env, "sns", &args).map(|stdout| {
        format!(
            "Deployed SNS:\nSNS config: {}\nCanister ID file: {}\n\n{}",
            path.display(),
            canister_ids_file.display(),
            stdout
        )
    })
}
