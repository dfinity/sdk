use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::expiry_duration;

use anyhow::bail;
use clap::Clap;
use slog::info;
use std::time::Duration;

/// Deletes a canister on the Internet Computer network.
#[derive(Clap)]
pub struct CanisterDeleteOpts {
    /// Specifies the name of the canister to delete.
    /// You must specify either a canister name or the --all flag.
    canister_name: Option<String>,

    /// Deletes all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister-name"))]
    all: bool,
}

async fn delete_canister(
    env: &dyn Environment,
    canister_name: &str,
    timeout: Duration,
) -> DfxResult {
    let mut canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id = canister_id_store.get(canister_name)?;
    info!(
        env.get_logger(),
        "Deleting code for canister {}, with canister_id {}",
        canister_name,
        canister_id.to_text(),
    );

    canister::delete_canister(env, canister_id, timeout).await?;

    canister_id_store.remove(canister_name)?;

    Ok(())
}

pub async fn exec(env: &dyn Environment, opts: CanisterDeleteOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let timeout = expiry_duration();

    fetch_root_key_if_needed(env).await?;

    if let Some(canister_name) = opts.canister_name.as_deref() {
        delete_canister(env, canister_name, timeout).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister_name in canisters.keys() {
                delete_canister(env, canister_name, timeout).await?;
            }
        }
        Ok(())
    } else {
        bail!("Cannot find canister name.")
    }
}
