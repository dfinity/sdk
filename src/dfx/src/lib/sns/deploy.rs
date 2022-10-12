//! Code for creating an SNS.
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::call_bundled::call_bundled;
use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;
use crate::Environment;

/// Creates an SNS.  This requires funds but no proposal.
#[context("Failed to deploy SNS with config: {}", path.display())]
pub fn deploy_sns(
    env: &dyn Environment,
    path: &Path,
    network: Option<String>,
) -> DfxResult<String> {
    let agent_environment = create_agent_environment(env, network)?;
    let network_descriptor = agent_environment.get_network_descriptor();
    let network_str = if network_descriptor.is_ic {
        "ic"
    } else if let Ok(url) = network_descriptor.first_provider() {
        url
    } else {
        "local"
    };
    let args = vec![
        OsString::from("deploy"),
        OsString::from("--network"),
        OsString::from(network_str),
        OsString::from("--init-config-file"),
        OsString::from(path),
    ];
    call_bundled(env, "sns", &args)
        .map(|stdout| format!("Deployed SNS: {}\n{}", path.display(), stdout))
}
