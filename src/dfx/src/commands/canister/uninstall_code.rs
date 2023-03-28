use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::operations::canister;
use dfx_core::network::root_key::fetch_root_key_if_needed;

use candid::Principal;
use clap::Parser;
use slog::info;
use tokio::runtime::Runtime;

/// Uninstalls a canister, removing its code and state.
/// Does not delete the canister.
#[derive(Parser)]
pub struct UninstallCodeOpts {
    /// Specifies the name or id of the canister to uinstall.
    /// You must specify either a canister name/id or the --all option.
    canister: Option<String>,

    /// Uninstalls all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,
}

async fn uninstall_code(
    env: &dyn Environment,
    canister: &str,
    call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    info!(
        log,
        "Uninstalling code for canister {}, with canister_id {}",
        canister,
        canister_id.to_text(),
    );

    canister::uninstall_code(env, canister_id, call_sender).await?;

    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: UninstallCodeOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow::anyhow!("Cannot get HTTP client from environment."))?;
    let network = env.get_network_descriptor();
    runtime.block_on(async { fetch_root_key_if_needed(&agent, &network).await })?;

    if let Some(canister) = opts.canister.as_deref() {
        uninstall_code(env, canister, call_sender).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                uninstall_code(env, canister, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
