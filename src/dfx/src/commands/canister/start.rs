use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use slog::info;

/// Starts a stopped canister.
#[derive(Parser)]
pub struct CanisterStartOpts {
    /// Specifies the name or id of the canister to start. You must specify either a canister name/id or the --all flag.
    canister: Option<String>,

    /// Starts all of the canisters configured in the dfx.json file.
    #[arg(long, required_unless_present("canister"))]
    all: bool,
}

async fn start_canister(
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
        "Starting code for canister {}, with canister_id {}",
        canister,
        canister_id.to_text(),
    );

    canister::start_canister(env, canister_id, call_sender).await?;

    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterStartOpts,
    call_sender: &CallSender,
) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    if let Some(canister) = opts.canister.as_deref() {
        start_canister(env, canister, call_sender).await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;
        let config_interface = config.get_config();
        let network = env.get_network_descriptor();
        if let Some(canisters) = &config_interface.canisters {
            for canister in canisters.keys() {
                let canister_is_remote =
                    config_interface.is_remote_canister(canister, &network.name)?;
                if canister_is_remote {
                    info!(
                        env.get_logger(),
                        "Skipping canister '{canister}' because it is remote for network '{}'",
                        &network.name,
                    );

                    continue;
                }

                start_canister(env, canister, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
