use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::expiry_duration;

use candid::Principal;
use clap::Parser;
use slog::info;
use std::time::Duration;

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
    timeout: Duration,
    call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    info!(
        log,
        "Uninstalling code for canister {}, with canister_id {}",
        canister,
        canister_id.to_text(),
    );

    canister::uninstall_code(env, canister_id, timeout, call_sender).await?;

    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: UninstallCodeOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;

    fetch_root_key_if_needed(env).await?;
    let timeout = expiry_duration();

    if let Some(canister) = opts.canister.as_deref() {
        uninstall_code(env, canister, timeout, call_sender).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                uninstall_code(env, canister, timeout, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
