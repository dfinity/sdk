use crate::commands::wallet::get_wallet;
use crate::lib::diagnosis::DiagnosedError;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::wallet::{set_wallet_id, GetOrCreateWalletCanisterError};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::{format_as_trillions, pretty_thousand_separators};
use anyhow::{anyhow, bail, Context};
use candid::{encode_args, CandidType, Decode, Deserialize, Principal};
use clap::Parser;
use ic_agent::Agent;
use ic_utils::interfaces::WalletCanister;
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

    /// Redeem coupon to a new cycles wallet, creates a if the identity does not have one, otherwise returns an error.
    #[arg(long, default_value = "false")]
    new_cycles_wallet: bool,
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
    let wallet = get_wallet(env)
        .await
        .map_err(|e| e.downcast::<GetOrCreateWalletCanisterError>());
    let coupon_code = opts.coupon_code;
    match wallet {
        Ok(_) if opts.new_cycles_wallet => {
            bail!("A cycles wallet already exists for the current identity. Use the wallet to redeem the coupon.");
        }
        // identity already has a wallet - faucet should top up the wallet
        Ok(wallet_canister) => {
            let redeemed_cycles =
                redeem_to_existing_wallet(agent, &wallet_canister, &faucet_principal, &coupon_code)
                    .await?;
            info!(
                log,
                "Redeemed coupon code {coupon_code} for {} TC (trillion cycles) to the existing wallet {}",
                pretty_thousand_separators(format_as_trillions(redeemed_cycles)),
                wallet_canister.canister_id_()
            );
        }
        // identity has no wallet yet - faucet will provide one
        Err(Ok(GetOrCreateWalletCanisterError::NoWalletConfigured { .. }))
            if opts.new_cycles_wallet =>
        {
            let (redeemed_cycles, new_wallet_address) =
                create_wallet_and_redeem(agent, env, &faucet_principal, &coupon_code).await?;
            info!(
                log,
                "Redeemed coupon {coupon_code} for {} TC (trillion cycles) to a new wallet {new_wallet_address}.",
                pretty_thousand_separators(format_as_trillions(redeemed_cycles))
            );
        }
        Err(_) if opts.new_cycles_wallet => {
            bail!("Failed to create a new cycles wallet.");
        }
        // identity has no wallet yet - faucet will redeem the coupon to the cycles ledger
        Err(_) => {
            let redeemed_cycles =
                redeem_to_cycles_ledger(agent, env, &faucet_principal, &coupon_code).await?;
            info!(
                log,
                "Redeemed coupon code {coupon_code} for {} TC (trillion cycles) to the cycles ledger.",
                pretty_thousand_separators(format_as_trillions(redeemed_cycles))
            );
        }
    };

    Ok(())
}

async fn redeem_to_existing_wallet(
    agent: &Agent,
    wallet_canister: &WalletCanister<'_>,
    faucet_principal: &Principal,
    coupon_code: &str,
) -> DfxResult<u128> {
    let wallet_principal = wallet_canister.canister_id_();
    let response = agent
        .update(&faucet_principal, "redeem_to_wallet")
        .with_arg(
            encode_args((coupon_code, wallet_principal))
                .context("Failed to serialize redeem_to_wallet arguments.")?,
        )
        .call_and_wait()
        .await
        .context("Failed redeem_to_wallet call.")?;
    let redeemed_cycles =
        Decode!(&response, u128).context("Failed to decode redeem_to_wallet response.")?;
    Ok(redeemed_cycles)
}

async fn create_wallet_and_redeem(
    agent: &Agent,
    env: &dyn Environment,
    faucet_principal: &Principal,
    coupon_code: &str,
) -> DfxResult<(u128, Principal)> {
    let identity = env
        .get_selected_identity()
        .with_context(|| anyhow!("No identity selected."))?;
    let response = agent
        .update(&faucet_principal, "redeem")
        .with_arg(encode_args((coupon_code,)).context("Failed to serialize 'redeem' arguments.")?)
        .call_and_wait()
        .await
        .context("Failed 'redeem' call.")?;
    let new_wallet_address =
        Decode!(&response, Principal).context("Failed to decode 'redeem' response.")?;
    set_wallet_id(env.get_network_descriptor(), &identity, new_wallet_address)
        .with_context(|| {
        DiagnosedError::new(
            format!(
                "dfx failed while trying to set your new wallet, '{}'",
                &new_wallet_address
            ),
            format!("Please save your new wallet's ID '{}' and set the wallet manually afterwards using 'dfx identity set-wallet'.", &new_wallet_address),
        )
    })?;
    let redeemed_cycles = WalletCanister::create(agent, new_wallet_address.clone())
        .await
        .unwrap()
        .wallet_balance()
        .await
        .unwrap()
        .amount;
    Ok((redeemed_cycles, new_wallet_address))
}

async fn redeem_to_cycles_ledger(
    agent: &Agent,
    env: &dyn Environment,
    faucet_principal: &Principal,
    coupon_code: &str,
) -> DfxResult<u128> {
    #[derive(CandidType, Deserialize)]
    struct Account {
        owner: Principal,
        subaccount: Option<Vec<u8>>,
    }
    let identity = env
        .get_selected_identity_principal()
        .with_context(|| anyhow!("No identity selected."))?;
    let response = agent
        .update(&faucet_principal, "redeem_to_cycles_ledger")
        .with_arg(
            encode_args((
                coupon_code,
                Account {
                    owner: identity,
                    subaccount: None,
                },
            ))
            .context("Failed to serialize 'redeem_to_cycles_ledger' arguments.")?,
        )
        .call_and_wait()
        .await
        .context("Failed 'redeem_to_cycles_ledger' call.")?;
    let result = Decode!(&response, (u128, u128))
        .context("Failed to decode 'redeem_to_cycles_ledger' response.")?;
    let redeemed_cycles = result.0;
    Ok(redeemed_cycles)
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
