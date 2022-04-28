use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_or_anyhow;
use crate::util::clap::validators::{cycle_amount_validator, trillion_cycle_amount_validator};
use crate::util::expiry_duration;

use anyhow::Context;
use clap::Parser;
use ic_types::Principal;
use slog::info;
use std::time::Duration;

const DEFAULT_CYCLES_TO_FABRICATE: u128 = 10_000_000_000_000_u128;

/// Local development only: Fabricate cycles out of thin air and deposit them into the specified canister(s).
#[derive(Parser)]
pub struct FabricateCyclesOpts {
    /// Specifies the amount of cycles to fabricate. Defaults to 10T cycles.
    #[clap(long, validator(cycle_amount_validator), conflicts_with("t"))]
    amount: Option<String>,

    /// Specifies the amount of trillion cycles to fabricate. Defaults to 10T cycles.
    #[clap(
        long,
        validator(trillion_cycle_amount_validator),
        conflicts_with("amount")
    )]
    t: Option<String>,

    /// Specifies the name or id of the canister to receive the cycles deposit.
    /// You must specify either a canister name/id or the --all option.
    #[clap(long)]
    canister: Option<String>,

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
    let canister_id_store =
        CanisterIdStore::for_env(env).context("Failed to load canister id store.")?;
    let canister_id = Principal::from_text(canister)
        .or_else(|_| canister_id_store.get(canister))
        .context(format!("Failed to determine canister id for {}.", canister))?;

    info!(log, "Fabricating {} cycles onto {}", cycles, canister,);

    canister::provisional_deposit_cycles(env, canister_id, timeout, call_sender, cycles)
        .await
        .context("Failed provisional deposit.")?;

    let status = canister::get_canister_status(env, canister_id, timeout, call_sender).await;
    if status.is_ok() {
        info!(
            log,
            "Fabricated {} cycles, updated balance: {} cycles",
            cycles,
            status.unwrap().cycles
        );
    } else {
        info!(log, "Fabricated {} cycles.", cycles);
    }

    Ok(())
}

pub async fn exec(env: &dyn Environment, opts: FabricateCyclesOpts) -> DfxResult {
    // amount has been validated by cycle_amount_validator
    let cycles = cycles_to_fabricate(&opts);

    fetch_root_key_or_anyhow(env).await?;

    let timeout = expiry_duration();

    if let Some(canister) = opts.canister.as_deref() {
        deposit_minted_cycles(env, canister, timeout, &CallSender::SelectedId, cycles).await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                deposit_minted_cycles(env, canister, timeout, &CallSender::SelectedId, cycles)
                    .await
                    .context("Failed to mint cycles.")?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}

fn cycles_to_fabricate(opts: &FabricateCyclesOpts) -> u128 {
    if let Some(cycles_str) = &opts.amount {
        //cycles_str is validated by cycle_amount_validator
        cycles_str.parse::<u128>().unwrap()
    } else if let Some(t_cycles_str) = &opts.t {
        //cycles_str is validated by trillion_cycle_amount_validator
        format!("{}000000000000", t_cycles_str)
            .parse::<u128>()
            .unwrap()
    } else {
        DEFAULT_CYCLES_TO_FABRICATE
    }
}
