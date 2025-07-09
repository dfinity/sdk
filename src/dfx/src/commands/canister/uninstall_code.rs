use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::operations::canister::skip_remote_canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use slog::info;

/// Uninstalls a canister, removing its code and state.
/// Does not delete the canister.
#[derive(Parser)]
pub struct UninstallCodeOpts {
    /// Specifies the name or id of the canister to uinstall.
    /// You must specify either a canister name/id or the --all option.
    canister: Option<String>,

    /// Uninstalls all of the canisters configured in the dfx.json file.
    #[arg(long, required_unless_present("canister"))]
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
    fetch_root_key_if_needed(env).await?;

    if let Some(canister) = opts.canister.as_deref() {
        uninstall_code(env, canister, call_sender).await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;

        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                if skip_remote_canister(env, canister)? {
                    continue;
                }
                uninstall_code(env, canister, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
