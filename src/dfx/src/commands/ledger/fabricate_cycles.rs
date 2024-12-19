use super::get_icpts_from_args;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::operations::canister;
use crate::lib::operations::canister::skip_remote_canister;
use crate::lib::root_key::fetch_root_key_or_anyhow;
use crate::util::clap::parsers::{cycle_amount_parser, e8s_parser, trillion_cycle_amount_parser};
use crate::util::currency_conversion::as_cycles_with_current_exchange_rate;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use fn_error_context::context;
use slog::info;

const DEFAULT_CYCLES_TO_FABRICATE: u128 = 10_000_000_000_000_u128;

/// Local development only: Fabricate cycles out of thin air and deposit them into the specified canister(s).
/// Can specify a number of ICP/e8s (which will be converted to cycles using the current exchange rate) or a number of cycles.
/// If no amount is specified, 10T cycles are added.
#[derive(Parser)]
pub struct FabricateCyclesOpts {
    /// Specifies the amount of cycles to fabricate.
    #[arg(
        long,
        value_parser = cycle_amount_parser,
        conflicts_with("t"),
        conflicts_with("amount"),
        conflicts_with("icp"),
        conflicts_with("e8s")
    )]
    cycles: Option<u128>,

    /// ICP to mint into cycles and deposit into destination canister
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[arg(
        long,
        conflicts_with("cycles"),
        conflicts_with("icp"),
        conflicts_with("e8s"),
        conflicts_with("t")
    )]
    amount: Option<ICPTs>,

    /// Specify ICP as a whole number, helpful for use in conjunction with `--e8s`
    #[arg(
        long,
        value_parser = e8s_parser,
        conflicts_with("amount"),
        conflicts_with("cycles"),
        conflicts_with("t")
    )]
    icp: Option<u64>,

    /// Specify e8s as a whole number, helpful for use in conjunction with `--icp`
    #[arg(
        long,
        value_parser = e8s_parser,
        conflicts_with("amount"),
        conflicts_with("cycles"),
        conflicts_with("t")
    )]
    e8s: Option<u64>,

    /// Specifies the amount of trillion cycles to fabricate.
    #[arg(
        long,
        value_parser = trillion_cycle_amount_parser,
        conflicts_with("amount")
    )]
    t: Option<u128>,

    /// Specifies the name or id of the canister to receive the cycles deposit.
    /// You must specify either a canister name/id or the --all option.
    #[arg(long, required_unless_present("all"))]
    canister: Option<String>,

    /// Deposit cycles to all of the canisters configured in the dfx.json file.
    #[arg(long, required_unless_present("canister"))]
    all: bool,
}

#[context("Failed to deposit {} cycles into canister '{}'.", cycles, canister)]
async fn deposit_minted_cycles(
    env: &dyn Environment,
    canister: &str,
    call_sender: &CallSender,
    cycles: u128,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    info!(log, "Fabricating {} cycles onto {}", cycles, canister,);

    canister::provisional_deposit_cycles(env, canister_id, call_sender, cycles).await?;

    let status = canister::get_canister_status(env, canister_id, call_sender).await;
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

    if let Some(canister) = opts.canister.as_deref() {
        deposit_minted_cycles(env, canister, &CallSender::SelectedId, cycles).await
    } else if opts.all {
        let config = env.get_config_or_anyhow()?;

        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                if skip_remote_canister(env, canister)? {
                    continue;
                }

                deposit_minted_cycles(env, canister, &CallSender::SelectedId, cycles).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}

#[context("Failed to determine amount of cycles to fabricate.")]
async fn cycles_to_fabricate(env: &dyn Environment, opts: &FabricateCyclesOpts) -> DfxResult<u128> {
    if let Some(cycles) = opts.cycles {
        Ok(cycles)
    } else if let Some(t_cycles) = opts.t {
        Ok(t_cycles)
    } else if opts.amount.is_some() || opts.icp.is_some() || opts.e8s.is_some() {
        let icpts = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;
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
