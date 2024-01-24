use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::icrc_subaccount_parser;
use crate::util::{format_as_trillions, pretty_thousand_separators};
use anyhow::{anyhow, bail, Context};
use candid::{encode_args, CandidType, Decode, Deserialize, Principal};
use clap::Parser;
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use slog::{info, warn};

pub const DEFAULT_FAUCET_PRINCIPAL: Principal =
    Principal::from_slice(&[0, 0, 0, 0, 1, 112, 0, 196, 1, 1]);

/// Redeem a code at the cycles faucet.
#[derive(Parser)]
pub struct RedeemFaucetCouponOpts {
    /// The coupon code to redeem at the faucet.
    coupon_code: String,

    /// Alternative faucet address. If not set, this uses the DFINITY faucet.
    #[arg(long)]
    faucet: Option<String>,

    /// Subaccount to redeem the coupon to.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    to_subaccount: Option<Subaccount>,
}

pub async fn exec(env: &dyn Environment, opts: RedeemFaucetCouponOpts) -> DfxResult {
    let log = env.get_logger();

    let faucet_principal = if let Some(alternative_faucet) = opts.faucet {
        let canister_id_store = env.get_canister_id_store()?;
        Principal::from_text(&alternative_faucet)
            .or_else(|_| canister_id_store.get(&alternative_faucet))?
    } else {
        DEFAULT_FAUCET_PRINCIPAL
    };
    let agent = env.get_agent();
    if fetch_root_key_if_needed(env).await.is_err() {
        bail!("Failed to connect to the local replica. Did you forget to use '--network ic'?");
    } else if !env.get_network_descriptor().is_ic {
        warn!(log, "Trying to redeem a wallet coupon on a local replica. Did you forget to use '--network ic'?");
    }

    info!(log, "Redeeming coupon. This may take up to 30 seconds...");
    let identity = env
        .get_selected_identity_principal()
        .with_context(|| anyhow!("No identity selected."))?;
    let response = agent
        .update(&faucet_principal, "redeem_to_cycles_ledger")
        .with_arg(
            encode_args((
                opts.coupon_code.clone(),
                Account {
                    owner: identity,
                    subaccount: opts.to_subaccount,
                },
            ))
            .context("Failed to serialize 'redeem_to_cycles_ledger' arguments.")?,
        )
        .call_and_wait()
        .await
        .context("Failed 'redeem_to_cycles_ledger' call.")?;
    #[derive(CandidType, Deserialize)]
    struct DepositResponse {
        balance: u128,
        block_index: u128,
    }
    let result = Decode!(&response, DepositResponse)
        .context("Failed to decode 'redeem_to_cycles_ledger' response.")?;
    let redeemed_cycles = result.balance;
    info!(
        log,
        "Redeemed coupon '{}' to the cycles ledger, current balance: {} TC (trillions of cycles) for identity '{}'.",
        opts.coupon_code.clone(),
        pretty_thousand_separators(format_as_trillions(redeemed_cycles)),
        env
        .get_selected_identity()
        .with_context(|| anyhow!("No identity selected."))?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faucet_canister_id() {
        assert_eq!(
            DEFAULT_FAUCET_PRINCIPAL,
            Principal::from_text("fg7gi-vyaaa-aaaal-qadca-cai").unwrap()
        );
    }
}
