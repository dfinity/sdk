use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_or_anyhow;
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::expiry_duration;

use clap::Parser;
use ic_types::Principal;
use slog::info;
use std::time::Duration;

/// Local development only: Fabricate cycles out of thin air and deposit them into the specified canister(s).
#[derive(Parser)]
pub struct FabricateCyclesOpts {
    /// Specifies the name or id of the canister to receive the cycles deposit.
    /// You must specify either a canister name/id or the --all option.
    canister: Option<String>,

    /// Specifies the amount of cycles to fabricate.
    #[clap(validator(cycle_amount_validator), default_value = "10000000000000")]
    cycles: String,

    /// Deposit cycles to all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,
}

async fn deposit_minted_cycles(
    env: &dyn Environment,
    canister: &str,
    timeout: Duration,
    call_sender: &CallSender,
    cycles: u128,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    info!(log, "Fabricating {} cycles onto {}", cycles, canister,);

    canister::provisional_deposit_cycles(env, canister_id, timeout, call_sender, cycles).await?;

    let status = canister::get_canister_status(env, canister_id, timeout, call_sender).await?;

    info!(
        log,
        "Fabricated {} cycles, updated balance: {} cycles", cycles, status.cycles
    );

    Ok(())
}

pub async fn exec(env: &dyn Environment, opts: FabricateCyclesOpts) -> DfxResult {
    // amount has been validated by cycle_amount_validator
    let cycles = opts.cycles.parse::<u128>().unwrap();

    fetch_root_key_or_anyhow(env).await?;

    let timeout = expiry_duration();

    if let Some(canister) = opts.canister.as_deref() {
        deposit_minted_cycles(env, canister, timeout, &CallSender::SelectedId, cycles).await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                deposit_minted_cycles(env, canister, timeout, &CallSender::SelectedId, cycles)
                    .await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
