use crate::lib::api_version::fetch_api_version;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_manager::IdentityManager;
use crate::lib::provider::create_agent_environment;

use clap::Clap;
use slog::info;
use tokio::runtime::Runtime;

/// Renames an existing identity.
#[derive(Clap)]
pub struct RenameOpts {
    /// The current name of the identity.
    from: String,

    /// The new name of the identity.
    to: String,
}

pub fn exec(env: &dyn Environment, opts: RenameOpts, network: Option<String>) -> DfxResult {
    let from = opts.from.as_str();
    let to = opts.to.as_str();

    let log = env.get_logger();
    info!(log, r#"Renaming identity "{}" to "{}"."#, from, to);

    let agent_env = create_agent_environment(env, network.clone())?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    let ic_api_version = runtime.block_on(async { fetch_api_version(&agent_env).await })?;

    let renamed_default = IdentityManager::new(env)?.rename(env, from, to, ic_api_version)?;

    info!(log, r#"Renamed identity "{}" to "{}"."#, from, to);
    if renamed_default {
        info!(log, r#"Now using identity: "{}"."#, to);
    }

    Ok(())
}
