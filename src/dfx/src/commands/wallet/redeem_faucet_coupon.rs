use crate::lib::diagnosis::DiagnosedError;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::Identity;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::util::{format_as_trillions, pretty_thousand_separators};
use crate::{commands::wallet::get_wallet, lib::waiter::waiter_with_exponential_backoff};

use anyhow::{anyhow, Context};
use candid::{encode_args, Decode, Principal};
use clap::Parser;
use slog::info;

const DEFAULT_FAUCET_PRINCIPAL: &str = "fg7gi-vyaaa-aaaal-qadca-cai";

/// Redeem a code at the cycles faucet.
#[derive(Parser)]
pub struct RedeemFaucetCouponOpts {
    /// The coupon code to redeem at the faucet.
    coupon_code: String,

    /// Alternative faucet address. If not set, this uses the DFINITY faucet.
    #[clap(long)]
    faucet: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: RedeemFaucetCouponOpts) -> DfxResult {
    let faucet_principal = if let Some(alternative_faucet) = opts.faucet {
        let canister_id_store = CanisterIdStore::for_env(env)?;
        Principal::from_text(&alternative_faucet)
            .or_else(|_| canister_id_store.get(&alternative_faucet))?
    } else {
        Principal::from_text(DEFAULT_FAUCET_PRINCIPAL).unwrap()
    };
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;
    let log = env.get_logger();

    let wallet = get_wallet(env).await;
    match wallet {
        // identity has a wallet already - faucet should top up the wallet
        Ok(wallet_canister) => {
            let wallet_principal = wallet_canister.canister_id_();
            let response = agent
                .update(&faucet_principal, "redeem_to_wallet")
                .with_arg(
                    encode_args((opts.coupon_code.clone(), wallet_principal))
                        .context("Failed to serialize redeem_to_wallet arguments.")?,
                )
                .call_and_wait(waiter_with_exponential_backoff())
                .await
                .context("Failed redeem_to_wallet call.")?;
            let redeemed_cycles =
                Decode!(&response, u128).context("Failed to decode redeem_to_wallet response.")?;
            info!(
                log,
                "Redeemed coupon code {} for {} TC (trillion cycles).",
                opts.coupon_code,
                pretty_thousand_separators(format_as_trillions(redeemed_cycles))
            );

            Ok(())
        }
        // identity has no wallet yet - faucet will provide one
        _ => {
            let identity = env
                .get_selected_identity()
                .with_context(|| anyhow!("No identity selected."))?;
            let response = agent
                .update(&faucet_principal, "redeem")
                .with_arg(
                    encode_args((opts.coupon_code.clone(),))
                        .context("Failed to serialize 'redeem' arguments.")?,
                )
                .call_and_wait(waiter_with_exponential_backoff())
                .await
                .context("Failed 'redeem' call.")?;
            let new_wallet_address =
                Decode!(&response, Principal).context("Failed to decode 'redeem' response.")?;
            info!(
                log,
                "Redeemed coupon {} for a new wallet: {}", opts.coupon_code, &new_wallet_address
            );
            Identity::set_wallet_id(env.get_network_descriptor(), identity, new_wallet_address)
                .with_context(|| {
                    DiagnosedError::new(
                        format!(
                            "dfx failed while trying to set your new wallet, '{}'",
                            &new_wallet_address
                        ),
                        format!("Please save your new wallet's ID '{}' and set the wallet manually afterwards using 'dfx identity set-wallet'.", &new_wallet_address),
                    )
                })?;
            info!(log, "New wallet set.");
            Ok(())
        }
    }
}
