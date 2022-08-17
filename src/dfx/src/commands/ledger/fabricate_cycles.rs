use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_or_anyhow;
use crate::util::clap::validators::{
    cycle_amount_validator, e8s_validator, icpts_amount_validator, trillion_cycle_amount_validator,
};
use crate::util::currency_conversion::as_cycles_with_current_exchange_rate;
use crate::util::expiry_duration;

use candid::Principal;
use clap::Parser;
use fn_error_context::context;
use slog::info;
use std::time::Duration;

use super::get_icpts_from_args;

const DEFAULT_CYCLES_TO_FABRICATE: u128 = 10_000_000_000_000_u128;

/// Local development only: Fabricate cycles out of thin air and deposit them into the specified canister(s).
/// Can specify a number of ICP/e8s (which will be converted to cycles using the current exchange rate) or a number of cycles.
/// If no amount is specified, 10T cycles are added.
#[derive(Parser)]
pub struct FabricateCyclesOpts {
    /// Specifies the amount of cycles to fabricate.
    #[clap(
        long,
        validator(cycle_amount_validator),
        conflicts_with("t"),
        conflicts_with("amount"),
        conflicts_with("icp"),
        conflicts_with("e8s")
    )]
    cycles: Option<String>,

    /// ICP to mint into cycles and deposit into destination canister
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[clap(
        long,
        validator(icpts_amount_validator),
        conflicts_with("cycles"),
        conflicts_with("icp"),
        conflicts_with("e8s"),
        conflicts_with("t")
    )]
    amount: Option<String>,

    /// Specify ICP as a whole number, helpful for use in conjunction with `--e8s`
    #[clap(
        long,
        validator(e8s_validator),
        conflicts_with("amount"),
        conflicts_with("cycles"),
        conflicts_with("t")
    )]
    icp: Option<String>,

    /// Specify e8s as a whole number, helpful for use in conjunction with `--icp`
    #[clap(
        long,
        validator(e8s_validator),
        conflicts_with("amount"),
        conflicts_with("cycles"),
        conflicts_with("t")
    )]
    e8s: Option<String>,

    /// Specifies the amount of trillion cycles to fabricate.
    #[clap(
        long,
        validator(trillion_cycle_amount_validator),
        conflicts_with("amount")
    )]
    t: Option<String>,

    /// Specifies the name or id of the canister to receive the cycles deposit.
    /// You must specify either a canister name/id or the --all option.
    #[clap(long, required_unless_present("all"))]
    canister: Option<String>,

    /// Deposit cycles to all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,
}

#[context("Failed to deposite {} cycles into canister '{}'.", cycles, canister)]
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
    let cycles = cycles_to_fabricate(env, &opts).await?;

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

#[context("Failed to determine amount of cycles to fabricate.")]
async fn cycles_to_fabricate(env: &dyn Environment, opts: &FabricateCyclesOpts) -> DfxResult<u128> {
    if let Some(cycles_str) = &opts.cycles {
        //cycles_str is validated by cycle_amount_validator. Therefore unwrap is safe
        Ok(cycles_str.parse::<u128>().unwrap())
    } else if let Some(t_cycles_str) = &opts.t {
        //t_cycles_str is validated by trillion_cycle_amount_validator. Therefore unwrap is safe
        Ok(format!("{}000000000000", t_cycles_str)
            .parse::<u128>()
            .unwrap())
    } else if opts.amount.is_some() || opts.icp.is_some() || opts.e8s.is_some() {
        let icpts = get_icpts_from_args(&opts.amount, &opts.icp, &opts.e8s)?;
        let cycles = as_cycles_with_current_exchange_rate(&icpts).await?;
        let log = env.get_logger();
        info!(
            log,
            "At the current exchange rate, {} e8s produces approximately {} cycles.",
            icpts.get_e8s().to_string(),
            cycles.to_string()
        );
        Ok(cycles)
    } else {
        Ok(DEFAULT_CYCLES_TO_FABRICATE)
    }
}
