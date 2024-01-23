use crate::commands::wallet::get_wallet;
use crate::lib::diagnosis::DiagnosedError;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::wallet::set_wallet_id;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::{format_as_trillions, pretty_thousand_separators};
use anyhow::{anyhow, bail, Context};
use candid::{encode_args, Decode, Principal};
use clap::Parser;
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
                .call_and_wait()
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
                .call_and_wait()
                .await
                .context("Failed 'redeem' call.")?;
            let new_wallet_address =
                Decode!(&response, Principal).context("Failed to decode 'redeem' response.")?;
            info!(
                log,
                "Redeemed coupon {} for a new wallet: {}", opts.coupon_code, &new_wallet_address
            );
            set_wallet_id(env.get_network_descriptor(), identity, new_wallet_address)
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
